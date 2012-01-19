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

void rustclang_getCursorUSR(CXCursor* cursor, CXString* string) {
  *string = clang_getCursorUSR(*cursor);
}

void rustclang_getCursorSpelling(CXCursor* cursor, CXString* string) {
  *string = clang_getCursorSpelling(*cursor);
}

void rustclang_getCursorDisplayName(CXCursor* cursor, CXString* string) {
  *string = clang_getCursorDisplayName(*cursor);
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

void rustclang_getTypedefDeclUnderlyingType(CXCursor* cursor, CXType* ty) {
  *ty = clang_getTypedefDeclUnderlyingType(*cursor);
}

void rustclang_getEnumDeclIntegerType(CXCursor* cursor, CXType* ty) {
  *ty = clang_getEnumDeclIntegerType(*cursor);
}

long long rustclang_getEnumConstantDeclValue(CXCursor* cursor) {
  return clang_getEnumConstantDeclValue(*cursor);
}

unsigned long long rustclang_getEnumConstantDeclUnsignedValue(CXCursor* cursor) {
  return clang_getEnumConstantDeclUnsignedValue(*cursor);
}

void rustclang_getCursorType(CXCursor* cursor, CXType* ty) {
  *ty = clang_getCursorType(*cursor);
}

void rustclang_getCursorResultType(CXCursor* cursor, CXType* ty) {
  *ty = clang_getCursorResultType(*cursor);
}

enum CXAvailabilityKind rustclang_getCursorAvailability(CXCursor* cursor) {
  return clang_getCursorAvailability(*cursor);
}

enum CXLanguageKind rustclang_getCursorLanguage(CXCursor* cursor) {
  return clang_getCursorLanguage(*cursor);
}

void rustclang_getCanonicalType(CXType* in_ty, CXType* out_ty) {
  *out_ty = clang_getCanonicalType(*in_ty);
}

unsigned rustclang_isConstQualified(CXType* ty) {
  return clang_isConstQualifiedType(*ty);
}

unsigned rustclang_isVolatileQualified(CXType* ty) {
  return clang_isVolatileQualifiedType(*ty);
}

unsigned rustclang_isRestrictQualified(CXType* ty) {
  return clang_isRestrictQualifiedType(*ty);
}

void rustclang_getPointeeType(CXType* ty, CXType* out_ty) {
  *out_ty = clang_getPointeeType(*ty);
}

void rustclang_getTypeDeclaration(CXType* ty, CXCursor* cursor) {
  *cursor = clang_getTypeDeclaration(*ty);
}

enum CXCallingConv rustclang_getFunctionTypeCallingConv(CXType* ty) {
  return clang_getFunctionTypeCallingConv(*ty);
}

unsigned rustclang_getNumArgTypes(CXType* ty) {
  return clang_getNumArgTypes(*ty);
}

void rustclang_getArgType(CXType* in_ty, unsigned i, CXType* out_ty) {
  *out_ty = clang_getArgType(*in_ty, i);
}

unsigned rustclang_isFunctionTypeVariadic(CXType* ty) {
  return clang_isFunctionTypeVariadic(*ty);
}

void rustclang_getElementType(CXType* ty, CXType* out_ty) {
  *out_ty = clang_getElementType(*ty);
}

unsigned rustclang_getNumElements(CXType* ty) {
  return clang_getNumElements(*ty);
}

void rustclang_getResultType(CXType* in_ty, CXType* out_ty) {
  *out_ty = clang_getResultType(*in_ty);
}

unsigned rustclang_isPODType(CXType* ty) {
  return clang_isPODType(*ty);
}

void rustclang_getArrayElementType(CXType* in_ty, CXType* out_ty) {
  *out_ty = clang_getArrayElementType(*in_ty);
}

unsigned rustclang_getArraySize(CXType* ty) {
  return clang_getArraySize(*ty);
}
