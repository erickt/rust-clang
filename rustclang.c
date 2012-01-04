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
} file_inclusion_data;

static void getInclusions(CXFile included_file,
                          CXSourceLocation* inclusion_stack,
                          unsigned include_len,
                          CXClientData client_data) {
  if (include_len != 0) {
    file_inclusion_data* data = (file_inclusion_data*)client_data;
    unsigned i = data->len;
    void* tmp;
    
    data->len += 1;

    tmp = realloc(data->inclusions, sizeof(file_inclusion) * data->len);
    assert(tmp != NULL);
    data->inclusions = (file_inclusion*)tmp;

    data->inclusions[i].included_file = included_file;
    CXSourceLocation* location = inclusion_stack;
    data->inclusions[i].location = inclusion_stack[0];
    data->inclusions[i].depth = include_len;
  }
}

void rustclang_getInclusions(CXTranslationUnit tu,
                             file_inclusion** inclusions,
                             unsigned* len) {
  file_inclusion_data* data =
    (file_inclusion_data*)malloc(sizeof(file_inclusion_data));
  data->inclusions = NULL;
  data->len = 0;

  clang_getInclusions(tu, getInclusions, data);

  *inclusions = data->inclusions;
  *len = data->len;

  free(data);
}

void rustclang_getExpansionLocation(CXSourceLocation* location,
                                    CXFile *file,
                                    unsigned *line,
                                    unsigned *column,
                                    unsigned *offset) {
  clang_getExpansionLocation(*location, file, line, column, offset);
}

enum CXCursorKind rustclang_getCursorKind(CXCursor* cursor) {
  return clang_getCursorKind(*cursor);
}

CXString rustclang_getCursorSpelling(CXCursor* cursor) {
  return clang_getCursorSpelling(*cursor);
}

CXString rustclang_getCursorDisplayName(CXCursor* cursor) {
  return clang_getCursorDisplayName(*cursor);
}

typedef struct {
  CXCursor* children;
  unsigned len;
} cursor_children_data;

enum CXChildVisitResult visitCursorChild(CXCursor child,
                                         CXCursor parent,
                                         CXClientData client_data) {
  assert(clang_Cursor_isNull(child) == 0);

  cursor_children_data* data = (cursor_children_data*)client_data;
  unsigned i = data->len;
  void* tmp;

  data->len += 1;

  tmp = realloc(data->children, sizeof(CXCursor) * data->len);
  assert(tmp != NULL);
  data->children = (CXCursor*)tmp;
  data->children[i] = child;

  return CXChildVisit_Continue;
}

void rustclang_visitChildren(CXCursor* parent,
                             CXCursor** children,
                             unsigned* len) {
  cursor_children_data* data =
    (cursor_children_data*)malloc(sizeof(cursor_children_data));
  data->children = NULL;
  data->len = 0;

  clang_visitChildren(*parent, visitCursorChild, data);

  *children = data->children;
  *len = data->len;

  free(data);
}
