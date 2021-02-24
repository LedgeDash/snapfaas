#!/usr/bin/env python3
""" Read in a report generated by running `trace-cmd record -e kvm [-e kvmmmu]`
and print latencies spent in the host, i.e. between `kvm_entry` and `kvm_exit`
"""
import argparse
import sys
import numpy as np
from collections import defaultdict

def parse_report(report,infile=None, kvmmmu=False, PAGE_SHIFT=12):
    latencies = defaultdict(list)
    counts = defaultdict(int)
    max_lat = defaultdict(int)
    min_lat = dict()
    ept_violation_latencies = defaultdict(list)
    gfn_latencies = defaultdict(list)
    mmios = defaultdict(list)

    def classify_latency(reason, lines, lat_us):
        if reason == 'EPT_VIOLATION':
            if (lines[0].split()[3] != 'kvm_page_fault:' or int(lines[0].split()[5], 16) > 0xd0000000):
                # mmio can cause EPT VIOLATION instead of EPT MISCONFIG
                reason += '-mmio'
                try:
                    mmios[int(lines[0].split()[5], 16)].append(lat_us)
                except ValueError:
                    print(lines[0])
                    raise ValueError
            # used to manually fix a ftrace report that does not follow the script's assumption
            try:
                error_code = int(lines[0].strip().split()[-1], 16)
            except ValueError:
                print(lines[0])
                raise ValueError
            ept_violation_latencies[error_code].append(lat_us)
            if kvmmmu:
                strs = lines[1].strip().split()
                gfn_latencies[int(strs[7], 16) >> PAGE_SHIFT].append((strs[7], error_code, strs[9], lat_us))
        if reason == 'EPT_MISCONFIG':
            mmios[int(lines[1].split()[7], 16)].append(lat_us)
        latencies[reason].append(lat_us)
        counts[reason] += 1
        if lat_us > max_lat[reason]:
            max_lat[reason] = lat_us
        if reason not in min_lat:
            min_lat[reason] = lat_us
        elif lat_us < min_lat[reason]:
            min_lat[reason] = lat_us

    def readfile(infile):
        linenum = 0
        for line in infile:
            linenum += 1
            strs = line.strip().split()
            if len(strs) > 3 and strs[3] == 'kvm_exit:':
                try:
                    reason = strs[5]
                except IndexError:
                    print(line)
                    raise IndexError
                start_s, start_us = [int(x) for x in strs[2][:-1].split('.')]
                lines = []
                for line in infile:
                    linenum += 1
                    strs = line.strip().split()
                    end_s, end_us = [int(x) for x in strs[2][:-1].split('.')]
                    if strs[3] == 'kvm_entry:':
                        classify_latency(reason, lines, (end_s - start_s) * 1000000 + end_us - start_us)
                        break
                    else:
                        lines.append(line)

    if infile is None:
        with open(report) as infile:
            readfile(infile)
    else:
        readfile(infile)

    return latencies, counts, max_lat, min_lat, ept_violation_latencies, gfn_latencies, mmios

def parse_and_print():
    parser = argparse.ArgumentParser()
    parser.add_argument('--ftrace_report', nargs=1, help='ftrace report generated by `trace-cmd record -e kvm` or `trace-cmd record -e kvm -e kvmmmu`', required=True)
    parser.add_argument('--hugepage', action='store_true', help='if present ftrace-report is generated with hugepage turned on', required=False)
    parser.add_argument('--kvmmmu', action='store_true', help='if present ftrace-report is generated with kvmmmu events traced', required=False)
    parser.add_argument('--verbose', action='store_true', help='if present print all latencies in chronological order', required=False)
    parser.add_argument('--histogram', action='store_true', help='if present print latency histogram', required=False)
    parser.add_argument('--mmio', action='store_true', help='if present print mmio exit latencies', required=False)
    # argument parsing
    args = parser.parse_args()
    kvmmmu = args.kvmmmu
    verbose = args.verbose
    histogram = args.histogram
    mmio = args.mmio
    PAGE_SHIFT = 12
    if args.hugepage:
        PAGE_SHIFT += 9

    latencies, counts, max_lat, min_lat, ept_violation_latencies, gfn_latencies, mmios = parse_report(args.ftrace_report[0])

    print('###exit reason | total_latency (us) | counts | mean (us) | max (us) | min (us) | std')
    k = 'EPT_VIOLATION'
    v = latencies[k]
    if counts[k]:
        print('\t'.join([k, str(sum(v)), str(counts[k]), str(sum(v)/counts[k]), str(max_lat[k]), str(min_lat[k]), str(np.std(v))]))
    else:
        print('\t'.join([k, "0", "0", "0", "0", "0", "0"]))
    del latencies[k]
    for k, v in sorted(latencies.items(), key=lambda x: x[0]):
        print('\t'.join([k, str(sum(v)), str(counts[k]), str(sum(v)/counts[k]), str(max_lat[k]), str(min_lat[k]), str(np.std(v))]))

    if histogram:
        histogram = defaultdict(int)
        # print page fault latency histogram
        print('###range | count')
        for v in latencies['EPT_VIOLATION']:
           histogram[v // 5] += 1
        for k, v in sorted(histogram.items(), key=lambda x: x[0]):
            print('[{}, {}) us: {}'.format(k * 5, (k + 1) * 5, v))

    if verbose:
        print('###EPT_VIOLATION code | handling latencies in chronological order')
        for k, v in sorted(ept_violation_latencies.items(), key=lambda x: x[0]):
            print(hex(k), v)

    if mmio:
        print('MMIO Address (hex) | counts | latencies (us, chronological order)')
        for mmio, latencies in sorted(mmios.items(), key=lambda x: x[0]):
            print(hex(mmio), len(latencies), latencies)

    if kvmmmu:
        print('\nguest page number | time ordered list of (gpa, error code, error string, latency in us)')
        for k, v in gfn_latencies.items():
            print('{}: [ {} ]'.format(k, ', '.join(['(0x{}, {}, {}, {})'.format(x[0], hex(x[1]), x[2], x[3]) for x in v])))

        count = 0
        for _, v in gfn_latencies.items():
            for _, _, error_string, _ in v:
                if 'W' in error_string.split('|'):
                    count += 1
                    break
        print('# of pages that have at least one write fault: {}'.format(count))

        read_before_write = defaultdict(int)
        for k, v in gfn_latencies.items():
            for _, _, error_string, _ in v:
                chars = error_string.split('|')
                if 'W' in chars:
                    break
                read_before_write[k] += 1
        for k, v in read_before_write.items():
            if v > 1:
                print(k, v)
        print('number of read faults before the 1st write faults: {}'.format(len(read_before_write)))

        fault_counts = defaultdict(int)
        for k, v in gfn_latencies.items():
            for _, _, error_string, _ in v:
                chars = error_string.split('|')
                if 'W' in chars:
                    fault_counts[1] += 1
                else:
                    fault_counts[0] += 1
        print('number of read faults: {}'.format(fault_counts[0]))
        print('number of write faults: {}'.format(fault_counts[1]))

if __name__ == '__main__':
    parse_and_print()
