#ifndef LIB_H

#define LIB_H

#include <stdint.h>

char* stringify_i32(int32_t src);
char* stringify_f64(double src);

char* cmd_symbols_replace(char* src, Vec syms, Vec strs);
char* exec(const char* cmd);

#endif
