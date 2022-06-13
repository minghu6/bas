#ifndef VEC_H

#define VEC_H

#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

typedef struct Vec *Vec;
typedef int32_t i32;
typedef double f64;
typedef void *ptr;

#define declare_vec_new(type) \
  Vec vec_new_##type(size_t cap);

#define declare_vec_push(type) \
  int vec_push_##type(Vec v, type val);

#define declare_vec_get(type) \
  type vec_get_##type(Vec v, size_t idx);

#define declare_vec_set(type) \
  type vec_set_##type(Vec v, size_t idx, type val)

#define declare_vec_insert(type) \
  int vec_insert_##type(Vec v, size_t idx, type val);

#define declare_vec_bsearch_by_cmp(type) \
  size_t vec_bsearch_by_cmp_##type(Vec v, type val, int (*cmp)(const void *pkey, const void *pelem), size_t *insert_pos);

#define declare_vec_sorted_insert(type) \
  int vec_sorted_insert_##type(Vec v, type val, int (*cmp)(const void *pkey, const void *pelem))

#define declare_vec_all(type)       \
  declare_vec_new(type);            \
  declare_vec_push(type);           \
  declare_vec_get(type);            \
  declare_vec_set(type);            \
  declare_vec_insert(type);         \
  declare_vec_bsearch_by_cmp(type); \
  declare_vec_sorted_insert(type);

declare_vec_all(i32);
declare_vec_all(f64);
declare_vec_all(ptr);

size_t vec_len(Vec v);
void vec_drop(Vec v);

#endif
