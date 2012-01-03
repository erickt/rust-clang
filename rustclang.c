#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <clang-c/Index.h>

typedef struct {
    CXFile included_file;
    CXSourceLocation location;
    unsigned depth;
} file_inclusion;

typedef struct {
  file_inclusion* inclusions;
  unsigned len;
} file_inclusions;

static void getInclusions(CXFile included_file,
                          CXSourceLocation* inclusion_stack,
                          unsigned include_len,
                          void* data) {
  if (include_len == 1) { // != 0) {
    file_inclusions* inc = (file_inclusions*)data;
    unsigned i = inc->len;
    void* tmp;
    
    inc->len += 1;

    tmp = realloc(inc->inclusions, sizeof(file_inclusion) * inc->len);
    assert(tmp != NULL);
    inc->inclusions = (file_inclusion*)tmp;

    inc->inclusions[i].included_file = included_file;
    CXSourceLocation* location = inclusion_stack;
    inc->inclusions[i].location = inclusion_stack[0];
    inc->inclusions[i].depth = include_len;

    //printf("inc: %p %p %u\n", location->ptr_data[0], location->ptr_data[1], location->int_data);
  }
}

void rustclang_getInclusions(CXTranslationUnit tu,
                             file_inclusion** inclusions,
                             unsigned* len) {
  file_inclusions* inc = (file_inclusions*)malloc(sizeof(file_inclusions));
  inc->inclusions = NULL;
  inc->len = 0;

  clang_getInclusions(tu, getInclusions, inc);

  *inclusions = inc->inclusions;
  *len = inc->len;

  free(inc);
}

void rustclang_getExpansionLocation(CXSourceLocation* location,
                                    CXFile *file,
                                    unsigned *line,
                                    unsigned *column,
                                    unsigned *offset) {
  clang_getExpansionLocation(*location, file, line, column, offset);
}
