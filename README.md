# 🌐️ WebFS
### Filesystem operations exposed over a HTTP server

## 💡️ Idea
The idea behind WebFS is to expose common filesystem operations to be accessible through HTTP requests. You can have 2 physical disks, in different parts of the world, sharing information and syncing. It can even implement RAID through software in this configuration, with slaves watching master's disks and copying their data to the disk or by simply receiving commands from a controller.

The current version of WebFS exposes a frontend that uses the API to access the disk as Google Drive would do, but simpler, way much simpler.

## 🛠️ How to Build/Install

You can use it with docker or bare metal.

### 🐋 Docker

1. Install docker:
```sh
curl -fsSL https://get.docker.com | sudo sh
```
2. Run it with the following command:
```sh
sudo docker run \
    -p ACCESS_PORT:3000 \
    --name webfs-jqv \
    -e DATA_DIR=/JQV \
    -e PORT=3000 \
    -v /mnt/data0/JQV:/JQV \
    lucasperovani/webfs
```
#### Explanation:
- ACCESS_PORT: The port to access the HTTP server
- DATA_DIR: The data directory to list/upload/download/delete files

### 👾 Bare metal
1. Download the source:
```sh
git clone https://github.com/lucasperovani/WebFS.git && cd WebFS
```
2. Compile it:
```sh
cargo build --release
```
3. Run it:
```sh
DATA_DIR=/your/file/folder; PORT=3000; ./target/webfs
```
#### Explanation:
- PORT: The port to access the HTTP server
- DATA_DIR: The data directory to list/upload/download/delete files

If you want a more detailed installation with explanation about mounting the disk right at linux boot or VPN to be able access it anywhere, go to [INSTALLATION.md](INSTALLATION.md)

## 🕹️ How to Use

Please, go to our Postman API documentation to check which endpoints to use:
[WebFS API Documentation](https://www.postman.com/planetary-firefly-988785/webfs/collection/c1ydxpm/webfs?action=share&creator=21227029)

### Frontend Usage

## 📝️ To Do
- [x] Split rust code in different files
- [ ] Delete moves the file to a delete directory and a cron deletes when over 30 days
- [ ] Add tests
- [ ] Add github CI to test before pull requests
- [x] Add github CI to export docker images on merges
- [ ] Improve upload and download files
- [ ] Implement disk information and usage routes
- [ ] Only add assets and frontend routes with env var
