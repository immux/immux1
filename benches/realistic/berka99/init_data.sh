mkdir -p data-raw
cd data-raw
curl -O -L http://sorry.vse.cz/~berka/challenge/pkdd1999/data_berka.zip
unzip data_berka.zip
rm data_berka.zip
