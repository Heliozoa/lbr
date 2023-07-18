#!/bin/bash
# Creates the license HTML with cargo-about.

set -eu

cargo about generate about.hbs > ./data/license.html
