// C ABI of the bada ime engine (engine/src/ffi.rs). JSON in, JSON out. Returned strings belong to
// the engine and stay valid until the next call on the same engine.
#ifndef BADA_H
#define BADA_H

#include <stddef.h>
#include <stdint.h>

typedef struct BadaData BadaData;
typedef struct BadaEngine BadaEngine;

const BadaData *bada_data_new(const uint8_t *ko, size_t ko_len, const uint8_t *en, size_t en_len,
                              const uint8_t *hanja, size_t hanja_len, const uint8_t *oecd, size_t oecd_len,
                              const uint8_t *ja, size_t ja_len, const uint8_t *kanji, size_t kanji_len);
void bada_data_free(const BadaData *data);

BadaEngine *bada_new(const BadaData *data);
void bada_free(BadaEngine *engine);

const char *bada_key(BadaEngine *engine, const char *event_json);
const char *bada_set_settings(BadaEngine *engine, const char *settings_json);
const char *bada_set_mode(BadaEngine *engine, uint32_t mode);
const char *bada_flush(BadaEngine *engine);
const char *bada_host_selection(BadaEngine *engine, const char *text);
const char *bada_catalog(BadaEngine *engine);

#endif
