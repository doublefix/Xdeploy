#!/usr/bin/env python

import json
import sys

def get_inventory(environment):
    inventories = {
        "dev": {
            "web": {
                "hosts": ["dev-web1.example.com", "dev-web2.example.com"],
                "vars": {
                    "http_port": 8080
                }
            },
            "db": {
                "hosts": ["dev-db1.example.com"],
                "vars": {
                    "db_port": 5432,
                    "db_user": "dev_admin"
                }
            }
        },
        "default": {
            "web": {
                "hosts": ["prod-web1.example.com", "prod-web2.example.com"],
                "vars": {
                    "http_port": 80
                }
            },
            "db": {
                "hosts": ["prod-db1.example.com", "prod-db2.example.com"],
                "vars": {
                    "db_port": 5432,
                    "db_user": "prod_admin"
                }
            }
        }
    }
    
    return inventories.get(environment, inventories["default"])

if __name__ == "__main__":
    if len(sys.argv) == 2:
        environment = sys.argv[1]
    else:
        environment = "default"
    
    inventory = get_inventory(environment)
    print(json.dumps(inventory, indent=2))
