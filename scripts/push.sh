#!/bin/bash
# Syncs /crates mirrors found under the /crates directory

set -e

CRATES_PATH="/crates"
MIRROR_PATH="/mirror"

VALID_PACKAGES_PATH=()
for package_path in "$CRATES_PATH"/*
do
    echo "INFO: Checking structure : $package_path..."

    IS_VALID_RUST_MIRROR_FOLDER=1
    for suspected_crate_path in "$package_path"/*
    do
        # A rust mirror structure can't include folders
        # whose names are greater than 2 characters.
        suspected_crate_dir=$(basename "$suspected_crate_path")
        if [ ${#suspected_crate_dir} -gt 2 ]; then
            echo "WARN: Invalid package detected : $package_path ! Will be skipped."
            IS_VALID_RUST_MIRROR_FOLDER=0
            break
        fi
    done
    if [ $IS_VALID_RUST_MIRROR_FOLDER -eq 1 ]; then
        VALID_PACKAGES_PATH+=("$package_path")
    fi
done

NB_CRATES_TO_BE_PUSHED=0
for valid_package in "${VALID_PACKAGES_PATH[@]}"
do
    NB_CRATES_TO_BE_PUSHED=$(( NB_CRATES_TO_BE_PUSHED + $(find "$valid_package" -iname '*.crate' | wc -l) ))
done
echo "INFO: Found ${#VALID_PACKAGES_PATH[@]} packages to be pushed including ${NB_CRATES_TO_BE_PUSHED} crates to be pushed"

echo "INFO: Starting sync of crate files to the mirror..."
for valid_package in "${VALID_PACKAGES_PATH[@]}"
do
    echo "INFO: Syncing $valid_package..."
    rsync -arqP "$valid_package/" "$MIRROR_PATH/crates/"
done
echo "INFO: Finished sync of crate files to the mirror."

echo "INFO: Starting sync of index..."
for valid_package in "${VALID_PACKAGES_PATH[@]}"
do
    echo "INFO: Syncing index for $valid_package..."
    package_slug=$(basename "$valid_package")
    find "$valid_package" -iname '*.crate' | while read crate_path
    do
        removal_slug="$CRATES_PATH/$package_slug/"
        crate_dirname=$(dirname "$crate_path") # Ex: /crates/1678289443/er/rn/errno-dragonfly/0.1.2
        crate_slug_dir=${crate_dirname#$removal_slug}
        crate_version=$(basename "$crate_slug_dir")
        crate_slug_dir_noversion=${crate_slug_dir%%/$crate_version}
        crate_name=$(basename "$crate_slug_dir_noversion")
        crate_index_path="$MIRROR_PATH/crates.io-index/$crate_slug_dir_noversion"
        crate_sha256=$(sha256sum "$crate_path" | cut -d ' ' -f 1)

        # TODO(flavienbwk): For the moment, any crate not currently in index will be added the list. However they currently don't include any deps[]. This at least allows the retrieval of the package from a client.
        crate_index='{"name":"'$crate_name'","vers":"'$crate_version'","deps":[],"cksum":"'$crate_sha256'","features":{},"yanked":false}'

        if [ -e "$crate_index_path" ]; then
            # Append a line if one with the current version does not exist
            if ! grep -q "\"vers\":\"$crate_version\"" "$crate_index_path"; then
                echo "$crate_index" >> "$crate_index_path"
            fi
        else
            # Create the file and add line with the current version does not exist
            echo "$crate_index" > "$crate_index_path"
        fi
        exit 0
    done
done
echo "INFO: Finished sync of index."
