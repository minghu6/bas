#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>

#include "../include/vec.h"
#include "../include/cstr.h"


char* stringify_i32(int32_t src) {
  // 1 sign + ( 31 bit fraction = 2^31 = 10^10 = 10),
  // = 11 using 8 * 2 = 16 bytes

  char* s = malloc(16);

  assert(s != NULL);

  sprintf(s, "%d", src);

  return s;
}

/*
* return Heap Alloc str (need to free it manually)
*/
char* stringify_f64(double src) {
  // 1 sign + ( 52 bit fraction = 2^52 = 10^16 = 16) + 1 dot,
  // = 18 using 8 * 3 = 24 bytes

  char* s = malloc(24);

  assert(s != NULL);

  sprintf(s, "%lf", src);

  return s;
}


#define _is_ident_head(c) \
  (('a' <= c && c <= 'z' || 'A' <= c && c <= 'Z' || c == '_') ? true : false)

#define _is_ident_nonhead(c) \
  ((_is_ident_head(c) || '0' <= c && c <= '9') ? true : false)

#define ps(prefix,s) printf("%s: %s\n", prefix, s);

int _match_str(Vec syms, char* sym) {
  for (int i = 0; i < vec_len(syms); i++) {
    if (strcmp(vec_get_ptr(syms, i), sym) == 0) return i;
  }
  return -1;
}

char* cmd_symbols_replace(char* src, Vec syms, Vec strs) {
  int start = 0;
  int end = 0;
  int state = 0; // 0 skip normal char
                 // 1 in symbol

  // Vec subs = vec_new_ptr(5);  // [start, end)
  CStr res = cstr_empty();

  int srclen = strlen(src);

  for (int i = 0; i < srclen + 1; i++) {
    if (state == 0 && src[i] == '$' && i+1 < srclen && _is_ident_head(src[i+1])) {
      start = i + 1;
      state = 1;
      cstr_push_slice(res, src + end, i - end);
    }
    else if (state == 1 && (!_is_ident_nonhead(src[i]) || i == srclen)) {
      end = i;
      state = 0;
      char* sym = strndup(src + start, end - start);
      int idx = -1;
      if ((idx = _match_str(syms, sym)) >= 0) {
        cstr_push(res, vec_get_ptr(strs, idx));
      }
      else {
        cstr_push_slice(res, src + start - 1, end - start + 1);
      }
      free(sym);
    }
  }

  // Empty return
  if (!cstr_len(res)) {
    return strdup(src);
  };

  return cstr_into(res);
}


char* exec(const char* cmd) {
  char buffer[128];

  CStr result = cstr_empty();
  FILE* pipe = popen(cmd, "r");

  while (fgets(buffer, sizeof buffer, pipe) != NULL) {
    cstr_push(result, buffer);
  }

  pclose(pipe);
  return cstr_into(result);
}
