#!/usr/bin/env bash


for FOLDER in *
do
	if [ -d "$FOLDER" ]; then
		cd $FOLDER
		if [ -d "target" ]; then
			rm -rf target;
		fi
		cd ../
	fi
done