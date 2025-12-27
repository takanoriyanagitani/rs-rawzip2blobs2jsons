#!/bin/bash

izip=./sample.d/input.zip

genzip(){
	echo "creating the sample input zip..."
	mkdir -p ./sample.d

	jq -c -n '{"helo":"wrld0"}' | gzip --fast > ./sample.d/hw1.json.gz
	jq -c -n '{"helo":"wrld1"}' | gzip --fast > ./sample.d/hw2.json.gz

	jq -c -n '{
		"helo":"wrld1",
		"long":"message",
	}' | gzip --fast > ./sample.d/hw3.json.gz

	find \
		./sample.d \
		-type f \
		-name '*.json.gz' |
		sort |
		zip \
			-0 \
			-@ \
			-T \
			-v \
			-o \
			"${izip}"

	echo
}

test -f "${izip}" || genzip

echo "listing the entries of the zip..."
unzip -lv "${izip}"
echo

echo "showing the jsons of the zip..."
unzip -p "${izip}" | zcat | jq -c
echo

ex1() {
	echo "Example 1: Processing all items in the zip..."
	cat "${izip}" |
		wazero \
			run \
			-timeout 10s \
			./rawzip2blobs2jsons.wasm \
			-- \
			--zip-size-max 1048576 \
			--zip-name input.zip \
			--item-size-max 37 \
			--verbose \
			--item-content-type application/json \
			--item-content-encoding gzip |
		jq -c
	echo
}

ex2() {
	echo "Example 2: Demonstrating failure when zip file exceeds --zip-size-max..."
	cat "${izip}" |
		wazero \
			run \
			-timeout 10s \
			./rawzip2blobs2jsons.wasm \
			-- \
			--zip-size-max 128 \
			--zip-name input.zip \
			--item-size-max 131072 \
			--item-content-type application/json \
			--item-content-encoding gzip \
			--verbose
	echo
}

ex1
ex2
