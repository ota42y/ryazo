# RYAZO

[Gyazo](https://github.com/gyazo/Gyazo) clone by rust

# get started
```
mkdir data
cargo install diesel_cli --no-default-features --features sqlite
export DATABASE_URL=./data/ryazo.db
diesel setup
docker run -v `pwd`/data:/app/data -p 8888:8888 -e SERVER_ADDRESS=localhost:8888 -e BIND_ADDRESS=0.0.0.0:8888 -e SAVE_FOLDER=/app/data -e DATABASE_URL=/app/data/ryazo.db -it ota42y/ryazo  /app/ryazo

# if you have favicon, set ./data/favicon.ico
```

# development

```
cargo install diesel_cli --no-default-features --features sqlite
export DATABASE_URL=./data/ryazo.db
diesel setup
export DATABASE_URL=./data/ryazo_test.db
diesel setup
diesel migration run

cargo test
```

# build development image
```
docker build -t ryazo_test .
```
