# orma <img src="../orma.svg" alt="orma logo" height=26/>

A PostgreSQL ORM wirtten in Rust language

## Introduction

When you feel the need to persist data as documents on PostgreSQL you often want a way to map these documens on structs.

If you have such needs and you are using PostgreSQL instead of other databases, it's probably because you also want all other cool stuff present in PostgreSQL.

**orma** is a special ORM for PostgreSQL written in Rust language developed just for you!

**orma** takes advantage of PostgreSQL JSONB data representation and doesn't give up the relational advantages of a RDBMS like PostgreSQL.

**orma** provides out of the box features for search and CRUD operation over your documents

**orma** is fast and easy to learn, with a very simple API.

## Testing orma

Orma is tested against a running instance of PostgreSQL.

A docker image is provided to run the tests.
If you don't want the docker image or you can't use it, or you need differet parameters,
you can set the following environment variables and test the library on your own environment.

- ORMA_DB_HOSTNAME (default: "localhost")
- ORMA_DB_PORT (default: 5433)
- ORMA_DB_NAME (default: "pgactix")
- ORMA_DB_USERNAME (default: "pgactix")
- ORMA_DB_PASSWORD (default: "pgactix")
