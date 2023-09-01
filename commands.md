## Tests

### Run tests with tracing output

Piped to `bunyan` to format nicely.

```sh
TEST_LOG=true cargo test | bunyan
```

## Digital Ocean

### List DO apps

This is to obtain `APP_ID` and endpoint URL

```sh
doctl apps list
```

### Update DO apps with app spec

```sh
doctl apps update APP_ID --spec=spec.yaml
```

## Migrating the production DB

```sh
DATABASE_URL=postgresql://newsletter:PASSWORD@app-16907a65-935e-42c0-bee7-ce8956587eaa-do-user-1723956-0.b.db.ondigitalocean.com:25060/newsletter?sslmode=require sqlx migrate run
```

## Testing API endpoints

### Production

```sh
curl -i -X POST \
 -d 'email=th2222222omas_mann22@hotmail.com&name=Tom' \
 https://zero2prod-df6qp.ondigitalocean.app/subscriptions \
 --verbose
```

```sh
curl https://zero2prod-df6qp.ondigitalocean.app/health_check --verbose
```

### Local

```sh
curl -i -X POST \
 -d 'email=th22222222omas_mann22@hotmail.com&name=Tom' \
 http://127.0.0.1:8000/subscriptions \
--verbose
```

```sh
curl http://127.0.0.1:8000/health_check --verbose
```
