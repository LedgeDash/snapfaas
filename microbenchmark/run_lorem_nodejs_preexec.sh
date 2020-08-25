#!/usr/bin/env bash

if [ $# -ne 2 ]; then
    echo 'usage: ./run_lorem_nodejs_preexec.sh START_INDEX NUMBER_OF_ROUNDS'
    exit 1
fi
startindex=$1
rounds=$(($1 + $2 - 1))

[ $(cat ./.stat | head -1) != 'setup' ] && echo 'Please run ./setup.sh before run this script.' && exit 1

source ./env

echo 'Starting...'
# drop page cache
echo 1 | sudo tee /proc/sys/vm/drop_caches &>/dev/null
outDir=lorem-diff-out
[ ! -d $outDir ] && mkdir $outDir
for ((i=$startindex; i<=$rounds; i++))
do
    echo "Round $i"
    taskset -c 0 sudo $MEMBINDIR/fc_wrapper \
	--vcpu_count 1 \
	--mem_size 128 \
	--kernel $KERNEL \
	--network 'tap0/aa:bb:cc:dd:ff:00' \
	--firerunner $MEMBINDIR/firerunner \
	--rootfs $SSDROOTFSDIR/nodejs-preexec.ext4 \
	--appfs $SSDAPPFSDIR/lorem-nodejs.ext2 \
	--load_dir $MEMPREEXECSNAPSHOTDIR/nodejs \
	--diff_dirs $SSDPREEXECSNAPSHOTDIR/diff/lorem-nodejs \
	--copy_diff > $outDir/lorem-nodejs.$i.txt < ../resources/requests/lorem-nodejs.json
    [ $? -ne 0 ] && echo '!! failed' && exit 1
done

outDir=lorem-fullapp-ondemand-out
echo 'Starting fullapp ondemand...'
[ ! -d $outDir ] && mkdir $outDir
for ((i=$startindex; i<=$rounds; i++))
do
    echo "Round $i"
    # drop page cache
    echo 1 | sudo tee /proc/sys/vm/drop_caches &>/dev/null
    cat ../resources/requests/lorem-nodejs.json | head -1 | \
    taskset -c 0 sudo $MEMBINDIR/fc_wrapper \
	--vcpu_count 1 \
	--mem_size 128 \
	--kernel $KERNEL \
	--network 'tap0/aa:bb:cc:dd:ff:00' \
	--firerunner $MEMBINDIR/firerunner \
	--rootfs $SSDROOTFSDIR/lorem-nodejs-preexec.ext4 \
	--load_dir $SSDPREEXECSNAPSHOTDIR/lorem-nodejs \
	> $outDir/lorem-nodejs.$i.txt
    [ $? -ne 0 ] && echo '!! failed' && exit 1
done

echo 'Starting fullapp eager...'
outDir=lorem-fullapp-eager-out
# drop page cache
echo 1 | sudo tee /proc/sys/vm/drop_caches &>/dev/null
[ ! -d $outDir ] && mkdir $outDir
for ((i=$startindex; i<=$rounds; i++))
do
    echo "Round $i"
    cat ../resources/requests/lorem-nodejs.json | head -1 | \
    taskset -c 0 sudo $MEMBINDIR/fc_wrapper \
	--vcpu_count 1 \
	--mem_size 128 \
	--kernel $KERNEL \
	--network 'tap0/aa:bb:cc:dd:ff:00' \
	--firerunner $MEMBINDIR/firerunner \
	--rootfs $SSDROOTFSDIR/lorem-nodejs-preexec.ext4 \
	--load_dir $SSDPREEXECSNAPSHOTDIR/lorem-nodejs \
	--copy_base \
	--odirect_base \
	> $outDir/lorem-nodejs.$i.txt
    [ $? -ne 0 ] && echo '!! failed' && exit 1
done

outDir=lorem-regular-out
# drop page cache
echo 1 | sudo tee /proc/sys/vm/drop_caches &>/dev/null
[ ! -d $outDir ] && mkdir $outDir
for ((i=$startindex; i<=$rounds; i++))
do
    echo "Round $i"
    cat ../resources/requests/lorem-nodejs.json | head -1 | \
    taskset -c 0 sudo $MEMBINDIR/fc_wrapper \
	--vcpu_count 1 \
	--mem_size 128 \
	--kernel $KERNEL \
	--network 'tap0/aa:bb:cc:dd:ff:00' \
	--firerunner $MEMBINDIR/firerunner \
	--rootfs $SSDROOTFSDIR/lorem-nodejs-preexec.ext4 \
	> $outDir/lorem-nodejs.$i.txt
    [ $? -ne 0 ] && echo '!! failed' && exit 1
done
