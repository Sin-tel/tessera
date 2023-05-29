----------- Warning, this file is autogenerated by cbindgen. Don't modify this manually. -----------
local ffi = require("ffi")
ffi.cdef([[




typedef struct C_Buffer {
  double *ptr;
  size_t len;
  size_t cap;
} C_Buffer;

typedef enum LuaMessage_Tag {
  Cpu,
  Meter,
} LuaMessage_Tag;

typedef struct Meter_Body {
  float _0;
  float _1;
} Meter_Body;

typedef struct LuaMessage {
  LuaMessage_Tag tag;
  union {
    struct {
      float cpu;
    };
    Meter_Body meter;
  };
} LuaMessage;

void add_channel(void *stream_ptr, size_t instrument_number);

void add_effect(void *stream_ptr, size_t channel_index, size_t effect_number);

void block_free(struct C_Buffer block);

struct C_Buffer get_spectrum(void *stream_ptr);

void pause(void *stream_ptr);

void play(void *stream_ptr);

struct C_Buffer render_block(void *stream_ptr);

bool rx_is_empty(void *stream_ptr);

struct LuaMessage rx_pop(void *stream_ptr);

void send_cv(void *stream_ptr, size_t ch, float pitch, float vel);

void send_mute(void *stream_ptr, size_t ch, bool mute);

void send_note_on(void *stream_ptr, size_t ch, float pitch, float vel, size_t id);

void send_pan(void *stream_ptr, size_t ch, float gain, float pan);

void send_param(void *stream_ptr, size_t ch_index, size_t device_index, size_t index, float value);

void stream_free(void *stream_ptr);

/**
 * # Safety
 *
 * Make sure the arguments point to valid null-terminated c strings.
 */
void *stream_new(const char *host_ptr, const char *device_ptr);

]])
