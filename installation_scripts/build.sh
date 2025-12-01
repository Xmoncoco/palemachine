#!/usr/bin/env bash
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
mkdir package
cargo build --release --manifest-path $SCRIPT_DIR/../Cargo.toml --target-dir ./package
cd package
cp -r $SCRIPT_DIR/../{.version,bambam_morigatsu_chuapo.sh,pages,downloader,requirement.txt,update.sh} ./

# Copy config.toml if it exists, otherwise copy example
if [ -f "$SCRIPT_DIR/../config.toml" ]; then
    cp "$SCRIPT_DIR/../config.toml" ./
elif [ -f "$SCRIPT_DIR/../config.toml.example" ]; then
    cp "$SCRIPT_DIR/../config.toml.example" ./config.toml
else
    echo "⚠️ No config.toml or config.toml.example found"
fi

python3 -m venv venv 
source ./venv/bin/activate
./venv/bin/pip install -r requirement.txt
deactivate
cp ./release/palemachine ./palemachine
rm -r ./release

# Copy .env only if it doesn't exist
if [ ! -f ".env" ]; then
    cp $SCRIPT_DIR/../env_exemple ./.env
    echo "⚠️ you need to set you credentials in the .env file"
else
    echo "✅ .env file preserved"
fi
