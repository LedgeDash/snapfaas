#!/usr/bin/env bash

echo 'Building app generic nodejs-preexec.ext4...'
cd ../snapfaas-images/separate
echo "switching to $PWD"
for runtime in nodejs-preexec
do
    echo "- $SSDROOTFSDIR/$runtime.ext4"
    ./mk_rtimage.sh $runtime $SSDROOTFSDIR/$runtime.ext4 &>/dev/null
    echo "- $MEMROOTFSDIR/$runtime.ext4"
    cp $SSDROOTFSDIR/$runtime.ext4 $MEMROOTFSDIR/$runtime.ext4
done

echo 'Building app specific root filesystems...'
cd ../combined
echo "switching to $PWD"
appfsDir='../appfs'
for language in nodejs
do
    apps=$(ls $appfsDir/$language)
    for app in $apps
    do
        echo "- $app-$language.ext4"
        ./mk_appimage.sh $language $SSDROOTFSDIR/$app-$language.ext4 $appfsDir/$language/$app &>/dev/null
    done
done

cd ../../microbenchmark
echo "swithing back to $PWD"
