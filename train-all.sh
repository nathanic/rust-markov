#!/bin/bash

order=$1
if [[ -z $order ]]; then
    order=5
fi
echo building a model file of order $order for all txt files in books/

for f in books/*.txt; do
    ./target/markov train $order alles${order}.db "$f"
    echo ---
done

echo all done!
