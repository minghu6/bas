#ifndef CSTR_H

#define CSTR_H

#include <stdint.h>
#include "vec.h"

typedef Vec CStr;

CStr cstr_empty();

CStr cstr_new(char* src);

size_t cstr_len(CStr v);

void cstr_push(CStr cstr, char* src);

void cstr_push_slice(CStr cstr, char* src, size_t nmbytes);

char* cstr_into(CStr cstr);

char* cstr_as_str(CStr cstr);

void cstr_drop(CStr cstr);

#endif
