#!/usr/bin/env python3
import json

REFERENCE_VALIDATOR = "5iZ5PQPy5Z9XDnkfoWPi6nvUgtxWnRFwZ36WaftPuaVM"
U64_MAX = 18446744073709551615

stake_total = 0
stake_accounts = 0

with open("tmp/sfdp.stake") as f:
    data = json.load(f)
    for entry in data["result"]:
        account = entry["account"]["data"]["parsed"]["info"]
        stake_info = account.get("stake")
        if not stake_info:
            continue
        delegation = stake_info.get("delegation")
        if not delegation:
            continue
        if delegation["voter"] != REFERENCE_VALIDATOR:
            continue
        if int(delegation["deactivationEpoch"]) != U64_MAX:
            continue
        stake = int(delegation["stake"])
        if stake > 0:
            stake_total += stake
            stake_accounts += 1

print(f"Validator: {REFERENCE_VALIDATOR}")
print(f"Active stake accounts: {stake_accounts}")
print(f"Total active stake (lamports): {stake_total}")
print(f"Total active stake (SOL): {stake_total / 1_000_000_000:.2f}")
