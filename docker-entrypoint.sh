#!/bin/sh
set -e

spangen "$@" | kcat -P -b "$KAFKA_BROKER" -t "$KAFKA_TOPIC" -p "${KAFKA_PARTITION:--1}"
