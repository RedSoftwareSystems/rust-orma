# Testing orma

Orma is tested against a running instance of PostgreSQL.

A docker image is provided to run the tests.
If you don't want the docker image or you can't use it, or you need differet parameters,
you can set the following environment variables and test the library on your own environment.

- ORMA_DB_HOSTNAME (default: "localhost")
- ORMA_DB_PORT (default: 5433)
- ORMA_DB_NAME (default: "pgactix")
- ORMA_DB_USERNAME (default: "pgactix")
- ORMA_DB_PASSWORD (default: "pgactix")
