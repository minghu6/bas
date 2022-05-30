#ifndef AA_H

#define AA_H

#include <stdlib.h>

typedef struct AssocArr* AssocArr;
typedef int32_t i32;
typedef double f64;
typedef void *ptr;

#define declare_aa_new(type)\
  AssocArr aa_new_##type(int cap);

#define declare_aa_insert(type)\
  int aa_insert_##type(AssocArr aa, const char *key, type val);

#define declare_aa_get_by_idx(type)\
  type aa_get_##type(AssocArr aa, size_t idx);

#define declare_aa_all(type)\
  declare_aa_new(type);\
  declare_aa_insert(type);\
  declare_aa_get_by_idx(type);


declare_aa_all(i32);
declare_aa_all(f64);
declare_aa_all(ptr);

int aa_search(AssocArr aa, const char *key, size_t* insert_pos);


#endif
