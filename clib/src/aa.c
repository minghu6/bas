/**
 * Associative Array
 *
 */
#include <stdlib.h>
#include <assert.h>
#include <string.h>
#include "../include/vec.h"

#include <stdio.h>
#define ps(s) printf("%s\n", s);

////////////////////////////////////////////////////////////////////////////////
/// Entry Vec

typedef struct Entry
{
  const char *key;
  size_t val;
} * Entry;

Entry _entry_new(const char *key, size_t val)
{
  Entry entry = calloc(1, sizeof(struct Entry));
  entry->key = key;
  entry->val = val;
  return entry;
}

void _entry_drop(Entry entry)
{
  free(entry);
}

// inline int _size_t_cmp(const void *a, const void *b)
// {
//   if (*(size_t *)(a) < *(size_t *)(b))
//     return -1;
//   if (*(size_t *)(a) == *(size_t *)(b))
//     return 0;
//   if (*(size_t *)(a) > *(size_t *)(b))
//     return 1;
// }
int _entry_cmp(const void *a, const void *b)
{
  // printf("_entry_cmp: %s, %s  \n", (*(Entry *)a)->key, (*(Entry *)b)->key);

  return strcmp((*(Entry *)a)->key, (*(Entry *)b)->key);
}

Vec _vec_new_entry(size_t cap)
{
  return vec_new_ptr(cap);
}

int _vec_insert_entry(Vec v, size_t idx, const char *key, size_t val)
{
  Entry entry = _entry_new(key, val);
  return vec_insert_ptr(v, idx, entry);
}

Entry _vec_get_entry(Vec v, size_t idx)
{
  return vec_get_ptr(v, idx);
}

int _vec_bsearch_entry(Vec v, const char *key, size_t *insert_pos)
{
  Entry fake_entry = _entry_new(key, 0);
  int res = vec_bsearch_by_cmp_ptr(v, fake_entry, _entry_cmp, insert_pos);
  _entry_drop(fake_entry);

  return res;
}

////////////////////////////////////////////////////////////////////////////////
/// Associative Aarray

typedef struct AssocArr
{
  Vec stridx;
  Vec vals;
} * AssocArr;

#define def_aa_new(type)                              \
  AssocArr aa_new_##type(int cap)                     \
  {                                                   \
    AssocArr aa = calloc(1, sizeof(struct AssocArr)); \
    aa->stridx = _vec_new_entry(cap);                 \
    aa->vals = vec_new_##type(cap);                   \
                                                      \
    return aa;                                        \
  }

#define def_aa_insert(type)                                                \
  int aa_insert_##type(AssocArr aa, const char *key, type val)             \
  {                                                                        \
    size_t insert_pos;                                                     \
    if (_vec_bsearch_entry(aa->stridx, key, &insert_pos) == 0)             \
    {                                                                      \
      Entry old_entry = _vec_get_entry(aa->stridx, insert_pos);            \
      assert(old_entry != NULL);                                           \
      vec_set_##type(aa->vals, old_entry->val, val);                       \
    }                                                                      \
    else                                                                   \
    {                                                                      \
      int res;                                                             \
      if ((res = _vec_insert_entry(aa->stridx, insert_pos, key, vec_len(aa->vals))) < 0) \
        return res;                                                        \
      if ((res = vec_push_##type(aa->vals, val)) < 0)                      \
        return res;                                                        \
    }                                                                      \
                                                                           \
    return 0;                                                              \
  }

/// if success return 0
/// else return negative value
int aa_search(AssocArr aa, const char *key, size_t* insert_pos)
{
  return _vec_bsearch_entry(aa->stridx, key, insert_pos);
}

#define def_aa_get_by_idx(type)                    \
  type aa_get_##type(AssocArr aa, size_t idx)      \
  {                                                \
    Entry entry = _vec_get_entry(aa->stridx, idx); \
    assert(entry != NULL);                         \
    return vec_get_##type(aa->vals, entry->val);   \
  }

#define def_aa_all(type)\
  def_aa_new(type);\
  def_aa_insert(type);\
  def_aa_get_by_idx(type);

def_aa_all(i32);
def_aa_all(f64);
def_aa_all(ptr);
