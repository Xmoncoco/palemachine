SCRIPT_DIR = $(dirname " &(readlink -f "$0")") 
echo("🆙 : NEW VERSION FOUND UPDATING")
mkdir $SCRIPT_DIR/../tempqddlkmfjqfejie
cp $SCRIPT_DIR/config.toml $SCRIPT_DIR/../tempqddlkmfjqfejie/config.toml
cp $SCRIPT_DIR/history_of_download.sqlite $SCRIPT_DIR/../tempqddlkmfjqfejie/history_of_download.sqlite
cp $SCRIPT_DIR/.env $SCRIPT_DIR/../tempqddlkmfjqfejie/.env
git clone  https://github.com/Xmoncoco/palemachine.git $SCRIPT_DIR/../palemachine
rm -rf $SCRIPT_DIR/*
cd ..
./palemachine/installation_scripts/build.sh
rm package/.env
cp -R ./tempqddlkmfjqfejie/* /package/
rm -R ./tempqddlkmfjqfejie
