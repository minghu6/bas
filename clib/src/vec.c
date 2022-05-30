#include <stdint.h>
#include <stddef.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <assert.h>

typedef struct Vec
{
  size_t len;
  size_t cap;
  void *data;
} * Vec;

typedef int32_t i32;
typedef double f64;
typedef void *ptr;

#include <stdio.h>
#define ps(s) printf("%s\n", s);

////////////////////////////////////////////////////////////////////////////////
/// Vec New

#define def_vec_new(type)                                    \
  Vec vec_new_##type(size_t cap)                             \
  {                                                          \
    if (cap == 0)                                            \
      return calloc(1, sizeof(struct Vec));                  \
                                                             \
    Vec v = malloc(sizeof (struct Vec));                     \
                                                             \
    v->len = 0;                                              \
    v->cap = cap;                                            \
    v->data = malloc(cap);                                   \
                                                             \
    return v;                                                \
  }

////////////////////////////////////////////////////////////////////////////////
/// Vec Push

#define def_vec_expand_capcity(type)                           \
  int vec_expand_capcity_##type(Vec v)                         \
  {                                                            \
    int new_cap = (v->cap == 0) ? 1 : v->cap << 1;             \
    assert(new_cap > v->len);                                  \
    void *new_data = realloc(v->data, sizeof(type) * new_cap); \
    if (new_data == NULL)                                      \
      return -1;                                               \
    v->data = new_data;                                        \
    v->cap = new_cap;                                          \
    return 0;                                                  \
  }

#define check_capcity(type, v)                                  \
  int res;                                                      \
  if (v->len >= v->cap && (res = vec_expand_capcity_##type(v))) \
    return res;

#define def_vec_push(type)               \
  int vec_push_##type(Vec v, type val)   \
  {                                      \
    check_capcity(type, v);              \
    ((type *)(v->data))[v->len++] = val; \
    return 0;                            \
  }

////////////////////////////////////////////////////////////////////////////////
/// Vec Get

#define def_vec_get(type)                \
  type vec_get_##type(Vec v, size_t idx) \
  {                                      \
    return ((type *)(v->data))[idx];     \
  }

////////////////////////////////////////////////////////////////////////////////
/// Vec Set

#define def_vec_set(type)                          \
  type vec_set_##type(Vec v, size_t idx, type val) \
  {                                                \
    return ((type *)(v->data))[idx] = val;         \
  }

////////////////////////////////////////////////////////////////////////////////
/// Vec Drop

// #define def_vec_drop(type)    \
//   void vec_drop_##type(Vec v) \
//   {                           \
//     free(v->data);            \
//     free(v);                  \
//   }

void vec_drop(Vec v)
{
  free(v->data);
  free(v);
}

////////////////////////////////////////////////////////////////////////////////
/// Vec Insert
#define def_vec_insert(type)                             \
  int vec_insert_##type(Vec v, size_t idx, type val)     \
  {                                                      \
    if (idx > v->len)                                    \
      return -1;                                         \
    if (idx == v->len)                                   \
      return vec_push_##type(v, val);                    \
                                                         \
    /* expand vec */                                     \
    check_capcity(type, v);                              \
                                                         \
    memmove(                                             \
        (type *)(v->data) + idx + 1,                     \
        (type *)(v->data) + idx,                         \
        (v->len - idx) * sizeof(type) /* [idx, v->len)*/ \
    );                                                   \
                                                         \
    *((type *)(v->data) + idx) = val;                    \
    v->len++;                                            \
    return 0;                                            \
  }

////////////////////////////////////////////////////////////////////////////////
/// Vec SortedInsert

/// Improve origin bsearch (return unmatched position)
/// if unmatch then return insertion postion

size_t _bsearch2(const void *key, const void *base,
                 size_t nmemb, size_t size,
                 int (*compar)(const void *pkey, const void *pelem))
{
  size_t l = 0;
  size_t h = nmemb; // [l, h)
  size_t pivot = 0;

  while (l < h)
  {
    pivot = (h + l) / 2;

    int cmp_res = compar(key, (uint8_t *)base + pivot * size);

    if (cmp_res < 0)
      h = pivot;
    else if (cmp_res == 0)
      break;
    else
      l = pivot + 1;
  }

  return pivot;
}

#define def_vec_sorted_insert(type)                                                              \
  int vec_sorted_insert_##type(Vec v, type val, int (*cmp)(const void *pkey, const void *pelem)) \
  {                                                                                              \
    type var_val = val;                                                                          \
    /* do insertion-sort on a sorted vec */                                                      \
    size_t insert_pos = _bsearch2(&var_val, v->data, v->len, sizeof(type), cmp);                 \
                                                                                                 \
    return vec_insert_##type(v, insert_pos, val);                                                \
  }

////////////////////////////////////////////////////////////////////////////////
/// Vec BSearch

#define def_vec_bsearch_by_cmp(type)                                                                                     \
  size_t vec_bsearch_by_cmp_##type(Vec v, type val, int (*cmp)(const void *pkey, const void *pelem), size_t *insert_pos) \
  {                                                                                                                      \
    type var_val = val;                                                                                                  \
    *insert_pos = _bsearch2(&var_val, v->data, v->len, sizeof(type), cmp);                                               \
    if (!v->len || *insert_pos == v->len || cmp(&var_val, &((type *)(v->data))[*insert_pos]))                            \
    {                                                                                                                    \
      return -1;                                                                                                         \
    }                                                                                                                    \
                                                                                                                         \
    return 0;                                                                                                            \
  }

////////////////////////////////////////////////////////////////////////////////
/// Vec Attributes Access

size_t vec_len(Vec v)
{
  return v->len;
}

////////////////////////////////////////////////////////////////////////////////
/// Define All

#define def_vec_all(type)       \
  def_vec_new(type);            \
  def_vec_expand_capcity(type); \
  def_vec_push(type);           \
  def_vec_get(type);            \
  def_vec_set(type);            \
  def_vec_insert(type);         \
  def_vec_bsearch_by_cmp(type); \
  def_vec_sorted_insert(type);  \

def_vec_all(i32);
def_vec_all(f64);
def_vec_all(ptr);
