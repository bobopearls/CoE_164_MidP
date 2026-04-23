# CoE_164_MidP

cargo run -- train --data data/dataset.csv --output model.mura --vocab-size 300 --lr 0.1 --epochs 500


cargo run -- classify --model model.mura --text "(insert text)" 
