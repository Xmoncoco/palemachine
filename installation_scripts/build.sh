#!/usr/bin/env bash
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
mkdir package
cargo build --release --manifest-path $SCRIPT_DIR/../Cargo.toml --target-dir ./package
cd package
cp -r $SCRIPT_DIR/../{bambam_morigatsu_chuapo.sh,config.toml,pages,downloader,requirement.txt} ./
python3 -m venv venv 
source ./venv/bin/activate
./venv/bin/pip install -r requirement.txt
deactivate
cp ./release/palemachine ./palemachine
rm -r ./release
cp $SCRIPT_DIR/../env_exemple ./.env
echo "⚠️ you need to set you credentials in the .env file"
