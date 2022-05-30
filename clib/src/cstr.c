#include <string.h>
#include <stdlib.h>
#include "../include/vec.h"


typedef Vec CStr;

CStr cstr_empty() {
  return vec_new_ptr(0);
}

void cstr_push(CStr cstr, char* src) {
  vec_push_ptr(cstr, strdup(src));
}

void cstr_push_slice(CStr cstr, char* src, size_t nmbytes) {
  vec_push_ptr(cstr, strndup(src, nmbytes));
}

CStr cstr_new(char* src) {
  Vec v = vec_new_ptr(1);
  cstr_push(v, src);

  return v;
}

size_t cstr_len(CStr v) {
  int cnt = 0;
  for (int i = 0; i < vec_len(v); i++) {
    cnt += strlen(vec_get_ptr(v, i));
  }
  return cnt;
}


/*
* Convert CStr into char*
*/
char* cstr_into(CStr cstr) {
  int total_len = cstr_len(cstr);

  char* news = calloc(total_len + 1, 1);
  char* p = news;

  for (int i = 0; i < vec_len(cstr); i++) {
    char* sub = vec_get_ptr(cstr, i);
    size_t sub_len = strlen(sub);
    memcpy(p, sub, sub_len);
    free(sub);
    p += sub_len;
  }

  free(cstr);

  return news;
}

char* cstr_as_str(CStr cstr) {
  int total_len = cstr_len(cstr);

  char* news = calloc(total_len + 1, 1);
  char* p = news;

  for (int i = 0; i < vec_len(cstr); i++) {
    char* sub = vec_get_ptr(cstr, i);
    size_t sub_len = strlen(sub);
    memcpy(p, sub, sub_len);
    p += sub_len;
  }

  return news;
}

void cstr_drop(CStr cstr) {
  for (int i = 0; i < vec_len(cstr); i++) {
    free(vec_get_ptr(cstr, i));
  }
}
