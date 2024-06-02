#!/bin/bash

nerdctl exec -it postgre-db-1 bash -c "psql -h localhost -p 5432 -U username -d postgres"