#!/usr/bin/env bash
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
mkdir package
cargo build --release --manifest-path $SCRIPT_DIR/../Cargo.toml --target-dir ./package
cd package
cp -r $SCRIPT_DIR/../{.version,bambam_morigatsu_chuapo,config.toml,pages,downloader,requirement.txt,update.sh} ./
uv venv venv 
source ./venv/bin/activate
uv pip install -r requirement.txt
deactivate
cp ./release/palemachine ./palemachine
rm -r ./release
cp $SCRIPT_DIR/../env_exemple ./.env
echo "⚠️ you need to set you credentials in the .env file"
