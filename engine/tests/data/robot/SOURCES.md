# Robot corpora: sources

Built by `engine/tools/build_robot_corpus.py` (download commands in the script); each file is an even
sample of 400 sentences, except the `*-common.*` files: each language's 1,000 most common words.

| File | Source | Terms |
|---|---|---|
| `en-us-law.txt` | United States Code, Title 17 (Copyright), 2023 edition, govinfo.gov | Public domain (17 U.S.C. §105: works of the US Government) |
| `en-uk-law.txt` | Human Rights Act 1998, legislation.gov.uk | Open Government Licence v3.0. Contains public sector information licensed under the Open Government Licence v3.0. |
| `en-tatoeba.txt` | Tatoeba English sentences (tatoeba.org) | CC BY 2.0 FR |
| `ko-law.txt` | 대한민국헌법 (제10호), 민법 (법률 제9650호), via ko.wikisource.org | 법령: 저작권법 제7조, 보호받지 못하는 저작물 |
| `ko-common-voice.txt` | Common Voice Korean sentences (Mozilla) | CC0 1.0 |
| `ja-law.tsv` | 日本国憲法, 民法 (明治二十九年法律第八十九号), e-Gov 法令API | 法令: 著作権法第13条, 権利の目的とならない著作物 |
| `ja-tatoeba.tsv` | Tatoeba Japanese sentences (tatoeba.org) | CC BY 2.0 FR |
| `en-common.txt` | The most common words of Tatoeba's English sentences | CC BY 2.0 FR |
| `ko-common.txt` | The most common words in `engine/data/ko-words.tsv` | as `engine/data/LICENSE-corpora.txt` |
| `ja-common.tsv` | The most common words of Tatoeba's Japanese sentences | CC BY 2.0 FR |

Japanese readings: SudachiPy with SudachiDict core (Apache-2.0); only the readings, as text.
