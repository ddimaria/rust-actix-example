# Rust/Actix Framework

[![Build Status](https://travis-ci.com/ddimaria/rust-actix-framework.svg?branch=master)](https://travis-ci.com/ddimaria/rust-actix-framework)

A web framework built upon Actix Web using the Rust language.

## Motivation

Actix Web is a fast, powerful web framework for building web applications in Rust.
This project aims to create ergonomic abstractions comparable to frameworks in
other languages while attempting to maintain the performance benefits of Actix.

## Features

- Actix 2.x HTTP Server
- Multi-Database Support (CockroachDB, Postgres, MySQL, Sqlite)
- JWT Support
- Async Caching Layer with a Simple API
- Public and Secure Static File Service
- Diesel Database Operations are Non-Blocking
- Filesystem Organized for Scale
- .env for Local Development
- Integrated Application State with a Simple API
- Lazy Static Config struct
- Built-in Healthcheck (includes cargo version info)
- Listeners configured for TDD
- Custom Errors and HTTP Payload/Json Validation
- Secure Argon2i Password Hashing
- CORS Support
- Unit and Integration Tests
- Test Coverage Reports
- Dockerfile for Running the Server in a Container
- TravisCI Integration

## Featured Packages

- `Argon2i`: Argon2i Password Hasning
- `actix-cors`: CORS Support
- `actix-identity`: User Authentication
- `actix-redis` and `redis-async`: Async Caching Layer
- `actix-web`: Actix Web Server
- `derive_more`: Error Formatting
- `diesel`: ORM that Operates on Several Databases
- `dotenv`: Configuration Loader (.env)
- `envy`: Deserializes Environment Variables into a Config Struct
- `jsonwebtoken`: JWT encoding/decoding
- `kcov`: Coverage Analysis
- `listenfd`: Listens for Filesystem Changes
- `rayon`: Parallelize
- `r2d2`: Database Connection Pooling
- `validator`: Validates incoming Json

## Installation

Clone the repo and cd into the repo:

```shell
git clone https://github.com/ddimaria/rust-actix-framework.git
cd rust-actix-framework
```

Copy over the example .env file:

```shell
cp .env.example .env
```

**IMPORTANT:** Change .env values for your setup, paying special attention to the salt and various keys.

After you set the `DATABASE` value in .env, you'll need it to match the `default` value in the `features` section in `Cargo.toml` with the `DATABASE` value in .env:

```toml
[features]
cockroach = []
mysql = []
postgres = []
sqlite = []
default = ["mysql"]
```

_note:_ Only supply a SINGLE database in the `default` array.

Next, you'll need to install the Diesel CLI:

```shell
cargo install diesel_cli
```

If you run into errors, see http://diesel.rs/guides/getting-started/

Now run the migrations via the Diesel CLI:

```shell
diesel migration run
```

## Running the Server

To startup the server:

```shell
cargo run
```

## Autoreloading

To startup the server and autoreload on code changes:

```shell
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

## Tests

Integration tests are in the `/src/tests` folder. There are helper functions
to make testing the API straightforward. For example, if we want to test the
`GET /api/v1/user` route:

```rust
  use crate::tests::helpers::tests::assert_get;

  #[test]
  async fn test_get_users() {
      assert_get("/api/v1/user").await;
  }
```

Using the Actix test server, the request is sent and the response is asserted
for a successful response:

`assert!(response.status().is_success());`

Similarly, to test a POST route:

```rust
use crate::handlers::user::CreateUserRequest;
use crate::tests::helpers::tests::assert_post;

#[test]
async fn test_create_user() {
    let params = CreateUserRequest {
        first_name: "Satoshi".into(),
        last_name: "Nakamoto".into(),
        email: "satoshi@nakamotoinstitute.org".into(),
    };
    assert_post("/api/v1/user", params).await;
}
```

### Running Tests

To run all of the tests:

```shell
cargo test
```

### Test Covearage

I created a repo on DockerHub that I'll update with each Rust version
(starting at 1.37), whose tags will match the Rust version.

In the root of the project:

```shell
docker run -it --rm --security-opt seccomp=unconfined --volume "${PWD}":/volume --workdir /volume ddimaria/rust-kcov:1.37 --exclude-pattern=/.cargo,/usr/lib,/src/main.rs,src/server.rs
```

_note: coverage takes a long time to run (up to 30 mins)._

You can view the HTML output of the report at `target/cov/index.html`

## Docker

To build a Docker image of the application:

```shell
docker build -t rust_actix_framework .
```

Once the image is built, you can run the container in port 3000:

```shell
docker run -it --rm --env-file=.env.docker -p 3000:3000 --name rust_actix_framework rust_actix_framework
```

## Public Static Files

Static files are served up from the `/static` folder.
Directory listing is turned off.
Index files are supported (`index.html`).

Example:

```shell
curl -X GET http://127.0.0.1:3000/test.html
```

## Secure Static Files

To serve static files to authenticated users only, place them in the `/static-secure` folder.
These files are referenced using the root-level `/secure` path.

Example:

```shell
curl -X GET http://127.0.0.1:3000/secure/test.html
```

## Application State

A shared, mutable hashmap is automatically added to the server. To invoke this data in a handler, simply add `data: AppState<'_, String>` to the function signature.

### Helper Functions

#### get\<T\>(data: AppState\<T\>, key: &str) -> Option\<T\>

Retrieves a copy of the entry in application state by key.

Example:

```rust
use create::state::get;

pub async fn handle(data: AppState<'_, String>) -> impl Responder {
  let key = "SOME_KEY";
  let value = get(data, key);
  assert_eq!(value, Some("123".to_string()));
}
```

#### set\<T\>(data: AppState\<T\>, key: &str, value: T) -> Option\<T\>

Inserts or updates an entry in application state.

Example:

```rust
use create::state::set;

pub async fn handle(data: AppState<'_, String>) -> impl Responder {
  let key = "SOME_KEY";
  let value = set(data, key, "123".into());
  assert_eq!(value, None)); // if this is an insert
  assert_eq!(value, Some("123".to_string())); // if this is an update
}
```

#### delete\<T\>(data: AppState\<T\>, key: &str) -> Option\<T\>

Deletes an entry in application state by key.

Example:

```rust
use create::state::get;

pub async fn handle(data: AppState<'_, String>) -> impl Responder {
  let key = "SOME_KEY";
  let value = delete(data, key);
  assert_eq!(value, None);
}
```

## Application Cache

Asynchronous access to redis is automatically added to the server if a value is provided for the `REDIS_URL` environment variable.
To invoke this data in a handler, simply add `cache: Cache` to the function signature.

### Helper Functions

#### get(cache: Cache, key: &str) -> Result<String, ApiError>

Retrieves a copy of the entry in the application cache by key.

Example:

```rust
use crate::cache::{get, Cache};

pub async fn handle(cache: Cache) -> impl Responder {
  let key = "SOME_KEY";
  let value = get(cache, key).await?;
  assert_eq!(value, "123");
}
```

#### set(cache: Cache, key: &str, value: &str) -> Result<String, ApiError>

Inserts or updates an entry in the application cache.

Example:

```rust
use crate::cache::{set, Cache};

pub async fn handle(cache: Cache) -> impl Responder {
  let key = "SOME_KEY";
  set(cache, key, "123").await?;
}
```

#### delete(cache: Cache, key: &str) -> Result<String, ApiError>

Deletes an entry in the application cache by key.

Example:

```rust
use crate::cache::{delete, Cache};

pub async fn handle(cache: Cache) -> impl Responder {
  let key = "SOME_KEY";
  delete(cache, key).await?;
}
```

## Non-Blocking Diesel Database Operations

When accessing a database via Diesel, operations block the main server thread.
This blocking can be mitigated by running the blocking code in a thread pool from within the handler.

Example:

```rust
pub async fn get_user(
    user_id: Path<Uuid>,
    pool: Data<PoolType>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = block(move || find(&pool, *user_id)).await?;
    respond_json(user)
}
```

Blocking errors are automatically converted into ApiErrors to keep the api simple:

```rust
impl From<BlockingError<ApiError>> for ApiError {
    fn from(error: BlockingError<ApiError>) -> ApiError {
        match error {
            BlockingError::Error(api_error) => api_error,
            BlockingError::Canceled => ApiError::BlockingError("Thread blocking error".into()),
        }
    }
}
```

## Endpoints

### Healthcheck

Determine if the system is healthy.

`GET /health`

#### Response

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

Example:

```shell
curl -X GET http://127.0.0.1:3000/health
```

### Login

`POST /api/v1/auth/login`

#### Request

| Param    | Type   | Description              | Required | Validations           |
| -------- | ------ | ------------------------ | :------: | --------------------- |
| email    | String | The user's email address |   yes    | valid email address   |
| password | String | The user's password      |   yes    | at least 6 characters |

```json
{
  "email": "torvalds@transmeta.com",
  "password": "123456"
}
```

#### Response

Header

```json
HTTP/1.1 200 OK
content-length: 118
content-type: application/json
set-cookie: auth=COOKIE_VALUE_HERE; HttpOnly; Path=/; Max-Age=1200
date: Tue, 15 Oct 2019 02:04:54 GMT
```

Json Body

```json
{
  "id": "0c419802-d1ef-47d6-b8fa-c886a23d61a7",
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

**When sending subsequent requests, create a header variable `cookie` with the value `auth=COOKIE_VALUE_HERE`**

### Logout

`GET /api/v1/auth/logout`

#### Response

`200 OK`

Example:

```shell
curl -X GET http://127.0.0.1:3000/api/v1/auth/logout
```

### Get All Users

`GET /api/v1/user`

#### Response

```json
[
  {
    "id": "a421a56e-8652-4da6-90ee-59dfebb9d1b4",
    "first_name": "Satoshi",
    "last_name": "Nakamoto",
    "email": "satoshi@nakamotoinstitute.org"
  },
  {
    "id": "c63d285b-7794-4419-bfb7-86d7bb3ff17d",
    "first_name": "Barbara",
    "last_name": "Liskov",
    "email": "bliskov@substitution.org"
  }
]
```

Example:

```shell
curl -X GET http://127.0.0.1:3000/api/v1/user
```

### Get a User

`GET /api/v1/user/{id}`

#### Request

| Param | Type | Description   |
| ----- | ---- | ------------- |
| id    | Uuid | The user's id |

#### Response

```json
{
  "id": "a421a56e-8652-4da6-90ee-59dfebb9d1b4",
  "first_name": "Satoshi",
  "last_name": "Nakamoto",
  "email": "satoshi@nakamotoinstitute.org"
}
```

Example:

```shell
curl -X GET http://127.0.0.1:3000/api/v1/user/a421a56e-8652-4da6-90ee-59dfebb9d1b4
```

#### Response - Not Found

`404 Not Found`

```json
{
  "errors": ["User c63d285b-7794-4419-bfb7-86d7bb3ff17a not found"]
}
```

### Create a User

`POST /api/v1/user`

#### Request

| Param      | Type   | Description              | Required | Validations           |
| ---------- | ------ | ------------------------ | :------: | --------------------- |
| first_name | String | The user's first name    |   yes    | at least 3 characters |
| last_name  | String | The user's last name     |   yes    | at least 3 characters |
| email      | String | The user's email address |   yes    | valid email address   |

```json
{
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

#### Response

```json
{
  "id": "0c419802-d1ef-47d6-b8fa-c886a23d61a7",
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

Example:

```shell
curl -X POST \
  http://127.0.0.1:3000/api/v1/user \
  -H 'Content-Type: application/json' \
  -d '{
    "first_name": "Linus",
    "last_name": "Torvalds",
    "email": "torvalds@transmeta.com"
}'
```

#### Response - Validation Errors

`422 Unprocessable Entity`

```json
{
  "errors": [
    "first_name is required and must be at least 3 characters",
    "last_name is required and must be at least 3 characters",
    "email must be a valid email"
  ]
}
```

### Update a User

`PUT /api/v1/{id}`

#### Request

Path

| Param | Type | Description   |
| ----- | ---- | ------------- |
| id    | Uuid | The user's id |

Body

| Param      | Type   | Description              | Required | Validations           |
| ---------- | ------ | ------------------------ | :------: | --------------------- |
| first_name | String | The user's first name    |   yes    | at least 3 characters |
| last_name  | String | The user's last name     |   yes    | at least 3 characters |
| email      | String | The user's email address |   yes    | valid email address   |

```json
{
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

#### Response

```json
{
  "id": "0c419802-d1ef-47d6-b8fa-c886a23d61a7",
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

Example:

```shell
curl -X PUT \
  http://127.0.0.1:3000/api/v1/user/0c419802-d1ef-47d6-b8fa-c886a23d61a7 \
  -H 'Content-Type: application/json' \
  -d '{
    "first_name": "Linus",
    "last_name": "Torvalds",
    "email": "torvalds@transmeta.com"
}'
```

#### Response - Validation Errors

`422 Unprocessable Entity`

```json
{
  "errors": [
    "first_name is required and must be at least 3 characters",
    "last_name is required and must be at least 3 characters",
    "email must be a valid email"
  ]
}
```

#### Response - Not Found

`404 Not Found`

```json
{
  "errors": ["User 0c419802-d1ef-47d6-b8fa-c886a23d61a7 not found"]
}
```

### Delete a User

`DELETE /api/v1/user/{id}`

#### Request

| Param | Type | Description   |
| ----- | ---- | ------------- |
| id    | Uuid | The user's id |

#### Response

```json
{
  "id": "a421a56e-8652-4da6-90ee-59dfebb9d1b4",
  "first_name": "Satoshi",
  "last_name": "Nakamoto",
  "email": "satoshi@nakamotoinstitute.org"
}
```

#### Response

`200 OK`

Example:

```shell
curl -X DELETE http://127.0.0.1:3000/api/v1/user/a421a56e-8652-4da6-90ee-59dfebb9d1b4
```

#### Response - Not Found

`404 Not Found`

```json
{
  "errors": ["User c63d285b-7794-4419-bfb7-86d7bb3ff17a not found"]
}
```

## License

This project is licensed under:

- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
