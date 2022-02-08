# get Node Template
git clone -b latest --depth 1 https://github.com/substrate-developer-hub/substrate-node-template


# Install Node.js
sudo apt-get install nodejs
sudo apt-get install npm
# Install Yarn
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update
sudo apt install yarn

#Front to End template
git clone -b latest --depth 1 https://github.com/substrate-developer-hub/substrate-front-end-template

yarn install
