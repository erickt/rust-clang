use std;
import str::sbuf;
import std::c_vec;
import std::io::{print, println};

// ---------------------------------------------------------------------------

#[link_args = "-L ../rust/build-make/llvm/x86_64-apple-darwin/Release+Asserts/lib"]
native mod clang {
    type CXIndex;
    type CXTranslationUnit;
    type CXFile;

    fn clang_getCString(++string: CXString) -> sbuf;
    fn clang_disposeString(++string: CXString);

    fn clang_getFileName(SFile: CXFile) -> CXString;

    fn clang_createIndex(excludeDeclarationsFromPCH: ctypes::c_int,
                         displayDiagnostics: ctypes::c_int) -> CXIndex;
    fn clang_disposeIndex(index: CXIndex);

    fn clang_parseTranslationUnit(
        CIdx: CXIndex,
        source_filename: sbuf,
        command_line_args: *sbuf,
        num_command_line_args: ctypes::c_int,
        unsaved_files: *CXUnsavedFile,
        num_unsaved_files: ctypes::unsigned,
        options: ctypes::unsigned) -> CXTranslationUnit;

    fn clang_disposeTranslationUnit(tu: CXTranslationUnit);

    fn clang_getTranslationUnitSpelling(tu: CXTranslationUnit) -> CXString;

    fn clang_getTranslationUnitCursor(tu: CXTranslationUnit) -> CXCursor;

    fn clang_getCursorKindSpelling(kind: ctypes::enum) -> CXString;
    fn clang_isDeclaration(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isReference(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isExpression(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isStatement(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isAttribute(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isInvalid(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isTranslationUnit(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isPreprocessing(kind: ctypes::enum) -> ctypes::unsigned;
    fn clang_isUnexposed(kind: ctypes::enum) -> ctypes::unsigned;

    fn clang_getTypeKindSpelling(kind: ctypes::enum) -> CXString;
}

#[link_args = "-L."]
native mod rustclang {
    fn rustclang_getInclusions(tu: clang::CXTranslationUnit,
                               &inclusions: *_file_inclusion,
                               &len: ctypes::unsigned);

    fn rustclang_getExpansionLocation(location: CXSourceLocation,
                                      &file: clang::CXFile,
                                      &line: ctypes::unsigned,
                                      &column: ctypes::unsigned,
                                      &offset: ctypes::unsigned);

    fn rustclang_visitChildren(parent: CXCursor,
                               &children: *CXCursor,
                               &len: ctypes::unsigned);

    // Work around bug #1402.
    fn rustclang_getCursorKind(cursor: CXCursor) -> ctypes::enum;
    fn rustclang_getCursorUSR(cursor: CXCursor, string: CXString);
    fn rustclang_getCursorSpelling(cursor: CXCursor, string: CXString);
    fn rustclang_getCursorDisplayName(cursor: CXCursor, string: CXString);

    fn rustclang_getCursorType(cursor: CXCursor, ty: CXType);
    fn rustclang_getCursorResultType(cursor: CXCursor, ty: CXType);

    fn rustclang_getCanonicalType(in_ty: CXType, ty: CXType);
    fn rustclang_isConstQualified(ty: CXType) -> ctypes::unsigned;
    fn rustclang_isVolatileQualified(ty: CXType) -> ctypes::unsigned;
    fn rustclang_isRestrictQualified(ty: CXType) -> ctypes::unsigned;
    fn rustclang_getPointeeType(in_ty: CXType, out_ty: CXType);
    fn rustclang_getTypeDeclaration(ty: CXType, cursor: CXCursor);
    fn rustclang_getResultType(in_ty: CXType, out_ty: CXType);
    fn rustclang_isPODType(ty: CXType) -> ctypes::unsigned;
    fn rustclang_getArrayElementType(in_ty: CXType, out_ty: CXType);
    fn rustclang_getArraySize(ty: CXType) -> ctypes::longlong;
}

// ---------------------------------------------------------------------------

type CXString = {
    data: *ctypes::void,
    private_flags: ctypes::unsigned,
};

fn empty_cxstring() -> CXString {
    { data: ptr::null(), private_flags: 0u as ctypes::unsigned }
}

type string = obj {
    fn to_str() -> str;
};

// CXString wrapper.
fn new_string(string: CXString) -> string {
    resource string_res(string: CXString) {
        clang::clang_disposeString(string);
    }

    obj string_obj(string: string_res) {
        fn to_str() -> str unsafe {
            str::from_cstr(clang::clang_getCString(*string))
        }
    }

    string_obj(string_res(string))
}

// ---------------------------------------------------------------------------

type CXSourceLocation = {
    // This should actually be an array of 2 void*, but we can't express that.
    // Hopefully we won't run into alignment issues in the meantime.
    ptr_data0: *ctypes::void,
    ptr_data1: *ctypes::void,
    int_data: ctypes::unsigned,
};

type expansion = {
    file: file,
    line: uint,
    column: uint,
    offset: uint,
};

type source_location = obj {
    fn expansion() -> expansion;
    fn to_str() -> str;
};

// CXSourceLocation wrapper.
obj new_source_location(location: CXSourceLocation) {
    fn expansion() -> expansion unsafe {
        let file = unsafe::reinterpret_cast(0u);
        let line = 0u32;
        let column = 0u32;
        let offset = 0u32;

        rustclang::rustclang_getExpansionLocation(location,
                file,
                line,
                column,
                offset);

        {
            file: new_file(file),
            line: line as uint,
            column: column as uint,
            offset: offset as uint,
        }
    }

    fn to_str() -> str {
        let e = self.expansion();

        #fmt("<source_location file %s, line %u, column %u>",
            e.file.filename().to_str(),
            e.line,
            e.column)
    }
}

// ---------------------------------------------------------------------------

type file = obj {
    fn filename() -> string;
};

// CXFile wrapper.
obj new_file(file: clang::CXFile) {
    fn filename() -> string {
        new_string(clang::clang_getFileName(file))
    }
}

// ---------------------------------------------------------------------------

type CXUnsavedFile = {
    Filename: sbuf,
    Contents: sbuf,
    Length: ctypes::ulong
};

// ---------------------------------------------------------------------------


type _file_inclusion = {
    included_file: clang::CXFile,
    location: CXSourceLocation,
    depth: uint
};

type file_inclusion = {
    included_file: file,
    location: source_location,
    depth: uint
};


// file_inclusion wrapper.
fn new_file_inclusion(fu: _file_inclusion) -> file_inclusion {
    let included_file = new_file(fu.included_file);
    let location = new_source_location(fu.location);
    { included_file: included_file, location: location, depth: fu.depth }
}

// ---------------------------------------------------------------------------

type CXCursor = {
    kind: ctypes::enum,
    xdata: ctypes::c_int,
    data0: *ctypes::void,
    data1: *ctypes::void,
    data2: *ctypes::void
};

fn empty_cxcursor() -> CXCursor {
    {
        kind: 0 as ctypes::enum,
        xdata: 0 as ctypes::c_int,
        data0: ptr::null(),
        data1: ptr::null(),
        data2: ptr::null(),
    }
}

// It sure would be nice if rust's objects supported recursive types. In the
// meantime, break up the recursion by inserting a tag into the chain.
tag cursor_tag = cursor;

type cursor = obj {
    fn kind() -> cursor_kind;
    fn USR() -> string;
    fn spelling() -> string;
    fn display_name() -> string;
    fn children() -> [cursor_tag];
    fn cursor_type() -> cursor_type_tag;
    fn result_type() -> cursor_type_tag;
};

obj new_cursor(cursor: CXCursor) {
    fn kind() -> cursor_kind {
        new_cursor_kind(rustclang::rustclang_getCursorKind(cursor))
    }

    fn USR() -> string {
        let string = empty_cxstring();
        rustclang::rustclang_getCursorUSR(cursor, string);
        new_string(string)
    }

    fn spelling() -> string {
        let string = empty_cxstring();
        rustclang::rustclang_getCursorSpelling(cursor, string);
        new_string(string)
    }

    fn display_name() -> string {
        let string = empty_cxstring();
        rustclang::rustclang_getCursorDisplayName(cursor, string);
        new_string(string)
    }

    fn children() -> [cursor_tag] unsafe {
        let len = 0u as ctypes::unsigned;
        let children = ptr::null::<CXCursor>();
        rustclang::rustclang_visitChildren(cursor, children, len);
        let len = len as uint;

        let cv = c_vec::create(
            unsafe::reinterpret_cast(children),
            len);

        let v = vec::init_fn({|i|
            cursor_tag(new_cursor(c_vec::get(cv, i)))
        }, len);

        // llvm handles cleaning up the inclusions for us, so we can
        // just let them leak.

        v
    }

    fn cursor_type() -> cursor_type_tag {
        let ty = empty_cxtype();
        rustclang::rustclang_getCursorType(cursor, ty);
        cursor_type_tag(new_cursor_type(ty))
    }

    fn result_type() -> cursor_type_tag {
        let ty = empty_cxtype();
        rustclang::rustclang_getCursorResultType(cursor, ty);
        cursor_type_tag(new_cursor_type(ty))
    }
}

// ---------------------------------------------------------------------------

const CXCursor_UnexposedDecl : uint = 1u;
const CXCursor_StructDecl : uint = 2u;
const CXCursor_UnionDecl : uint = 3u;
const CXCursor_ClassDecl : uint = 4u;
const CXCursor_EnumDecl : uint = 5u;
const CXCursor_FieldDecl : uint = 6u;
const CXCursor_EnumConstantDecl : uint = 7u;
const CXCursor_FunctionDecl : uint = 8u;
const CXCursor_VarDecl : uint = 9u;
const CXCursor_ParmDecl : uint = 10u;
const CXCursor_ObjCInterfaceDecl : uint = 11u;
const CXCursor_ObjCCategoryDecl : uint = 12u;
const CXCursor_ObjCProtocolDecl : uint = 13u;
const CXCursor_ObjCPropertyDecl : uint = 14u;
const CXCursor_ObjCIvarDecl : uint = 15u;
const CXCursor_ObjCInstanceMethodDecl : uint = 16u;
const CXCursor_ObjCClassMethodDecl : uint = 17u;
const CXCursor_ObjCImplementationDecl : uint = 18u;
const CXCursor_ObjCCategoryImplDecl : uint = 19u;
const CXCursor_TypedefDecl : uint = 20u;
const CXCursor_CXXMethod : uint = 21u;
const CXCursor_Namespace : uint = 22u;
const CXCursor_LinkageSpec : uint = 23u;
const CXCursor_Constructor : uint = 24u;
const CXCursor_Destructor : uint = 25u;
const CXCursor_ConversionFunction : uint = 26u;
const CXCursor_TemplateTypeParameter : uint = 27u;
const CXCursor_NonTypeTemplateParameter : uint = 28u;
const CXCursor_TemplateTemplateParameter : uint = 29u;
const CXCursor_FunctionTemplate : uint = 30u;
const CXCursor_ClassTemplate : uint = 31u;
const CXCursor_ClassTemplatePartialSpecialization : uint = 32u;
const CXCursor_NamespaceAlias : uint = 33u;
const CXCursor_UsingDirective : uint = 34u;
const CXCursor_UsingDeclaration : uint = 35u;
const CXCursor_TypeAliasDecl : uint = 36u;
const CXCursor_ObjCSynthesizeDecl : uint = 37u;
const CXCursor_ObjCDynamicDecl : uint = 38u;
const CXCursor_CXXAccessSpecifier : uint = 39u;
const CXCursor_FirstDecl : uint = 1u; // CXCursor_UnexposedDecl;
const CXCursor_LastDecl : uint = 39u; // CXCursor_CXXAccessSpecifier;

const CXCursor_FirstRef : uint = 40u;
const CXCursor_ObjCSuperClassRef : uint = 40u;
const CXCursor_ObjCProtocolRef : uint = 41u;
const CXCursor_ObjCClassRef : uint = 42u;
const CXCursor_TypeRef : uint = 43u;
const CXCursor_CXXBaseSpecifier : uint = 44u;
const CXCursor_TemplateRef : uint = 45u;
const CXCursor_NamespaceRef : uint = 46u;
const CXCursor_MemberRef : uint = 47u;
const CXCursor_LabelRef : uint = 48u;
const CXCursor_OverloadedDeclRef : uint = 49u;
const CXCursor_LastRef : uint = 49u; // CXCursor_OverloadedDeclRef;

const CXCursor_FirstInvalid : uint = 70u;
const CXCursor_InvalidFile : uint = 70u;
const CXCursor_NoDeclFound : uint = 71u;
const CXCursor_NotImplemented : uint = 72u;
const CXCursor_InvalidCode : uint = 73u;
const CXCursor_LastInvalid : uint = 73u; // CXCursor_InvalidCode;

const CXCursor_FirstExpr : uint = 100u;
const CXCursor_UnexposedExpr : uint = 100u;
const CXCursor_DeclRefExpr : uint = 101u;
const CXCursor_MemberRefExpr : uint = 102u;
const CXCursor_CallExpr : uint = 103u;
const CXCursor_ObjCMessageExpr : uint = 104u;
const CXCursor_BlockExpr : uint = 105u;
const CXCursor_IntegerLiteral : uint = 106u;
const CXCursor_FloatingLiteral : uint = 107u;
const CXCursor_ImaginaryLiteral : uint = 108u;
const CXCursor_StringLiteral : uint = 109u;
const CXCursor_CharacterLiteral : uint = 110u;
const CXCursor_ParenExpr : uint = 111u;
const CXCursor_UnaryOperator : uint = 112u;
const CXCursor_ArraySubscriptExpr : uint = 113u;
const CXCursor_BinaryOperator : uint = 114u;
const CXCursor_CompoundAssignOperator : uint = 115u;
const CXCursor_ConditionalOperator : uint = 116u;
const CXCursor_CStyleCastExpr : uint = 117u;
const CXCursor_CompoundLiteralExpr : uint = 118u;
const CXCursor_InitListExpr : uint = 119u;
const CXCursor_AddrLabelExpr : uint = 120u;
const CXCursor_StmtExpr : uint = 121u;
const CXCursor_GenericSelectionExpr : uint = 122u;
const CXCursor_GNUNullExpr : uint = 123u;
const CXCursor_CXXStaticCastExpr : uint = 124u;
const CXCursor_CXXDynamicCastExpr : uint = 125u;
const CXCursor_CXXReinterpretCastExpr : uint = 126u;
const CXCursor_CXXConstCastExpr : uint = 127u;
const CXCursor_CXXFunctionalCastExpr : uint = 128u;
const CXCursor_CXXTypeidExpr : uint = 129u;
const CXCursor_CXXBoolLiteralExpr : uint = 130u;
const CXCursor_CXXNullPtrLiteralExpr : uint = 131u;
const CXCursor_CXXThisExpr : uint = 132u;
const CXCursor_CXXThrowExpr : uint = 133u;
const CXCursor_CXXNewExpr : uint = 134u;
const CXCursor_CXXDeleteExpr : uint = 135u;
const CXCursor_UnaryExpr : uint = 136u;
const CXCursor_ObjCStringLiteral : uint = 137u;
const CXCursor_ObjCEncodeExpr : uint = 138u;
const CXCursor_ObjCSelectorExpr : uint = 139u;
const CXCursor_ObjCProtocolExpr : uint = 140u;
const CXCursor_ObjCBridgedCastExpr : uint = 141u;
const CXCursor_PackExpansionExpr : uint = 142u;
const CXCursor_SizeOfPackExpr : uint = 143u;
const CXCursor_LastExpr : uint = 143u; // CXCursor_SizeOfPackExpr;

const CXCursor_FirstStmt : uint = 200u;
const CXCursor_UnexposedStmt : uint = 200u;
const CXCursor_LabelStmt : uint = 201u;
const CXCursor_CompoundStmt : uint = 202u;
const CXCursor_CaseStmt : uint = 203u;
const CXCursor_DefaultStmt : uint = 204u;
const CXCursor_IfStmt : uint = 205u;
const CXCursor_SwitchStmt : uint = 206u;
const CXCursor_WhileStmt : uint = 207u;
const CXCursor_DoStmt : uint = 208u;
const CXCursor_ForStmt : uint = 209u;
const CXCursor_GotoStmt : uint = 210u;
const CXCursor_IndirectGotoStmt : uint = 211u;
const CXCursor_ContinueStmt : uint = 212u;
const CXCursor_BreakStmt : uint = 213u;
const CXCursor_ReturnStmt : uint = 214u;
const CXCursor_AsmStmt : uint = 215u;
const CXCursor_ObjCAtTryStmt : uint = 216u;
const CXCursor_ObjCAtCatchStmt : uint = 217u;
const CXCursor_ObjCAtFinallyStmt : uint = 218u;
const CXCursor_ObjCAtThrowStmt : uint = 219u;
const CXCursor_ObjCAtSynchronizedStmt : uint = 220u;
const CXCursor_ObjCAutoreleasePoolStmt : uint = 221u;
const CXCursor_ObjCForCollectionStmt : uint = 222u;
const CXCursor_CXXCatchStmt : uint = 223u;
const CXCursor_CXXTryStmt : uint = 224u;
const CXCursor_CXXForRangeStmt : uint = 225u;
const CXCursor_SEHTryStmt : uint = 226u;
const CXCursor_SEHExceptStmt : uint = 227u;
const CXCursor_SEHFinallyStmt : uint = 228u;
const CXCursor_NullStmt : uint = 230u;
const CXCursor_DeclStmt : uint = 231u;
const CXCursor_LastStmt : uint = 231u; // CXCursor_DeclStmt;

const CXCursor_TranslationUnit : uint = 300u;

const CXCursor_FirstAttr : uint = 400u;
const CXCursor_UnexposedAttr : uint = 400u;
const CXCursor_IBActionAttr : uint = 401u;
const CXCursor_IBOutletAttr : uint = 402u;
const CXCursor_IBOutletCollectionAttr : uint = 403u;
const CXCursor_CXXFinalAttr : uint = 404u;
const CXCursor_CXXOverrideAttr : uint = 405u;
const CXCursor_AnnotateAttr : uint = 406u;
const CXCursor_LastAttr : uint = 406u; // CXCursor_AnnotateAttr;

const CXCursor_PreprocessingDirective : uint = 500u;
const CXCursor_MacroDefinition : uint = 501u;
const CXCursor_MacroExpansion : uint = 502u;
const CXCursor_MacroInstantiation : uint = 502u; // CXCursor_MacroExpansion;
const CXCursor_InclusionDirective : uint = 503u;
const CXCursor_FirstPreprocessing : uint = 500u; // CXCursor_PreprocessingDirective;
const CXCursor_LastPreprocessing : uint = 503u; // CXCursor_InclusionDirective;

type cursor_kind = obj {
    fn to_uint() -> uint;
    fn spelling() -> string;

    fn is_declaration() -> bool;
    fn is_reference() -> bool;
    fn is_expression() -> bool;
    fn is_statement() -> bool;
    fn is_attribute() -> bool;
    fn is_invalid() -> bool;
    fn is_translation_unit() -> bool;
    fn is_preprocessing() -> bool;
    fn is_unexposed() -> bool;
    fn is_exposed() -> bool;
};

obj new_cursor_kind(kind: ctypes::enum) {
    fn to_uint() -> uint { kind as uint }

    fn spelling() -> string {
        new_string(clang::clang_getCursorKindSpelling(kind))
    }

    fn is_declaration() -> bool {
        clang::clang_isDeclaration(kind) != 0u as ctypes::unsigned
    }

    fn is_reference() -> bool {
        clang::clang_isReference(kind) != 0u as ctypes::unsigned
    }

    fn is_expression() -> bool {
        clang::clang_isExpression(kind) != 0u as ctypes::unsigned
    }

    fn is_statement() -> bool {
        clang::clang_isStatement(kind) != 0u as ctypes::unsigned
    }

    fn is_attribute() -> bool {
        clang::clang_isAttribute(kind) != 0u as ctypes::unsigned
    }

    fn is_invalid() -> bool {
        clang::clang_isInvalid(kind) != 0u as ctypes::unsigned
    }

    fn is_translation_unit() -> bool {
        clang::clang_isTranslationUnit(kind) != 0u as ctypes::unsigned
    }

    fn is_preprocessing() -> bool {
        clang::clang_isPreprocessing(kind) != 0u as ctypes::unsigned
    }

    fn is_unexposed() -> bool {
        clang::clang_isUnexposed(kind) != 0u as ctypes::unsigned
    }

    fn is_exposed() -> bool {
        !self.is_unexposed()
    }
}

// ---------------------------------------------------------------------------

/**
* Reprents an invalid type (e.g., where no type is available).
*/
const CXType_Invalid : uint = 0u;

/**
* A type whose specific kind is not exposed via this interface.
*/
const CXType_Unexposed : uint = 1u;

/* Builtin types */
const CXType_Void : uint = 2u;
const CXType_Bool : uint = 3u;
const CXType_Char_U : uint = 4u;
const CXType_UChar : uint = 5u;
const CXType_Char16 : uint = 6u;
const CXType_Char32 : uint = 7u;
const CXType_UShort : uint = 8u;
const CXType_UInt : uint = 9u;
const CXType_ULong : uint = 10u;
const CXType_ULongLong : uint = 11u;
const CXType_UInt128 : uint = 12u;
const CXType_Char_S : uint = 13u;
const CXType_SChar : uint = 14u;
const CXType_WChar : uint = 15u;
const CXType_Short : uint = 16u;
const CXType_Int : uint = 17u;
const CXType_Long : uint = 18u;
const CXType_LongLong : uint = 19u;
const CXType_Int128 : uint = 20u;
const CXType_Float : uint = 21u;
const CXType_Double : uint = 22u;
const CXType_LongDouble : uint = 23u;
const CXType_NullPtr : uint = 24u;
const CXType_Overload : uint = 25u;
const CXType_Dependent : uint = 26u;
const CXType_ObjCId : uint = 27u;
const CXType_ObjCClass : uint = 28u;
const CXType_ObjCSel : uint = 29u;
const CXType_FirstBuiltin : uint = 2u; // CXType_Void;
const CXType_LastBuiltin : uint = 29u; // CXType_ObjCSel,

const CXType_Complex : uint = 100u;
const CXType_Pointer : uint = 101u;
const CXType_BlockPointer : uint = 102u;
const CXType_LValueReference : uint = 103u;
const CXType_RValueReference : uint = 104u;
const CXType_Record : uint = 105u;
const CXType_Enum : uint = 106u;
const CXType_Typedef : uint = 107u;
const CXType_ObjCInterface : uint = 108u;
const CXType_ObjCObjectPointer : uint = 109u;
const CXType_FunctionNoProto : uint = 110u;
const CXType_FunctionProto : uint = 111u;
const CXType_ConstantArray : uint = 112u;

type cursor_type_kind = obj {
    fn to_uint() -> uint;
    fn spelling() -> string;
};

obj new_cursor_type_kind(kind: ctypes::enum) {
    fn to_uint() -> uint {
        kind as uint
    }

    fn spelling() -> string {
        new_string(clang::clang_getTypeKindSpelling(kind))
    }
}

// ---------------------------------------------------------------------------

type CXType = {
    kind: ctypes::enum,
    data0: *ctypes::void,
    data1: *ctypes::void
};

fn empty_cxtype() -> CXType {
    {
        kind: 0 as ctypes::enum,
        data0: ptr::null(),
        data1: ptr::null()
    }
}

type cursor_type = obj {
    fn kind() -> cursor_type_kind;
    fn canonical_type() -> cursor_type_tag;
    fn is_const_qualified() -> bool;
    fn is_volatile_qualified() -> bool;
    fn is_restrict_qualified() -> bool;
    fn pointee_type() -> cursor_type_tag;
    fn type_declaration() -> cursor;
    fn result_type() -> cursor_type_tag;
    fn is_pod_type() -> bool;
    fn array_element_type() -> cursor_type_tag;
    fn array_size() -> u64;
};

tag cursor_type_tag = cursor_type;

obj new_cursor_type(ty: CXType) {
    fn kind() -> cursor_type_kind {
        new_cursor_type_kind(ty.kind)
    }

    fn canonical_type() -> cursor_type_tag {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getCanonicalType(ty, out_ty);
        cursor_type_tag(new_cursor_type(out_ty))
    }

    fn is_const_qualified() -> bool {
        rustclang::rustclang_isConstQualified(ty) != 0 as ctypes::unsigned
    }

    fn is_volatile_qualified() -> bool {
        rustclang::rustclang_isVolatileQualified(ty) != 0 as ctypes::unsigned
    }

    fn is_restrict_qualified() -> bool {
        rustclang::rustclang_isRestrictQualified(ty) != 0 as ctypes::unsigned
    }

    fn pointee_type() -> cursor_type_tag {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getPointeeType(ty, out_ty);
        cursor_type_tag(new_cursor_type(out_ty))
    }

    fn type_declaration() -> cursor {
        let cursor = empty_cxcursor();
        rustclang::rustclang_getTypeDeclaration(ty, cursor);
        new_cursor(cursor)
    }

    fn result_type() -> cursor_type_tag {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getResultType(ty, out_ty);
        cursor_type_tag(new_cursor_type(out_ty))
    }

    fn is_pod_type() -> bool {
        rustclang::rustclang_isPODType(ty) != 0 as ctypes::unsigned
    }

    fn array_element_type() -> cursor_type_tag {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getArrayElementType(ty, out_ty);
        cursor_type_tag(new_cursor_type(out_ty))
    }

    fn array_size() -> u64 {
        rustclang::rustclang_getArraySize(ty) as u64
    }
}

// ---------------------------------------------------------------------------

const CXTranslationUnit_None : uint = 0x0u;
const CXTranslationUnit_DetailedPreprocessingRecord : uint = 0x01u;
const CXTranslationUnit_Incomplete : uint = 0x02u;
const CXTranslationUnit_PrecompiledPreamble : uint = 0x04u;
const CXTranslationUnit_CacheCompletionResults : uint = 0x08u;
const CXTranslationUnit_CXXPrecompiledPreamble : uint = 0x10u;
const CXTranslationUnit_CXXChainedPCH : uint = 0x20u;
const CXTranslationUnit_NestedMacroExpansions : uint = 0x40u;
const CXTranslationUnit_NestedMacroInstantiations : uint = 0x40u;

type translation_unit = obj {
    fn spelling() -> string;
    fn inclusions() -> [file_inclusion];
    fn cursor() -> cursor;
};

// CXTranslationUnit wrapper.
fn new_translation_unit(tu: clang::CXTranslationUnit) -> translation_unit {
    resource translation_unit_res(tu: clang::CXTranslationUnit) {
        clang::clang_disposeTranslationUnit(tu);
    }

    obj translation_unit_obj(tu: translation_unit_res) {
        fn spelling() -> string {
            new_string(clang::clang_getTranslationUnitSpelling(*tu))
        }

        fn inclusions() -> [file_inclusion] unsafe {
            // We can't support the native clang_getInclusions, because it
            // needs a callback function which rust doesn't support.
            // Instead we'll make a vector in our stub library and copy the
            // values from it.

            let len = 0u as ctypes::unsigned;
            let inclusions = ptr::null::<_file_inclusion>();
            rustclang::rustclang_getInclusions(*tu, inclusions, len);
            let len = len as uint;

            let cv = c_vec::create(
                unsafe::reinterpret_cast(inclusions),
                len);

            let v = vec::init_fn(
                {|i| new_file_inclusion(c_vec::get(cv, i)) },
                len);

            // llvm handles cleaning up the inclusions for us, so we can
            // just let them leak.

            v
        }

        fn cursor() -> cursor {
            new_cursor(clang::clang_getTranslationUnitCursor(*tu))
        }
    }

    translation_unit_obj(translation_unit_res(tu))
}

// ---------------------------------------------------------------------------

type index = obj {
    fn parse(str, [str], [CXUnsavedFile], uint) -> translation_unit;
};

fn index(excludeDecls: bool) -> index {
    let excludeDeclarationsFromPCH = if excludeDecls { 1 } else { 0 };
    let index = clang::clang_createIndex(
        excludeDeclarationsFromPCH as ctypes::c_int,
        0 as ctypes::c_int);

    // CXIndex wrapper.
    resource index_res(index: clang::CXIndex) {
        clang::clang_disposeIndex(index);
    }

    obj index_obj(index: index_res) {
        fn parse(path: str,
                 args: [str],
                 unsaved_files: [CXUnsavedFile],
                 options: uint) -> translation_unit {
            // Work around bug #1400.
            let path = @path;

            // Note: we need to hold on tho these vector references while we
            // hold a pointer to their buffers
            let args = vec::map(args, {|arg| @arg });
            let argv = vec::map(args, {|arg|
                str::as_buf(*arg, { |buf| buf })
            });

            let tu =
                unsafe {
                    clang::clang_parseTranslationUnit(
                        *index,
                        str::as_buf(*path, { |buf| buf }),
                        vec::to_ptr(argv),
                        vec::len(argv) as ctypes::c_int,
                        vec::to_ptr(unsaved_files),
                        vec::len(unsaved_files) as ctypes::unsigned,
                        options as ctypes::unsigned)
                };

            new_translation_unit(tu)
        }
    };

    index_obj(index_res(index))
}

#[cfg(test)]
mod tests {
    fn print_children(cursor: cursor) {
        fn f(cursor: cursor, depth: uint) {
            if cursor.kind().is_declaration() {
                let ty = cursor.cursor_type();
                uint::range(0u, depth, { |_i| print(">") });
                println(#fmt("> [%u %s] <%s> <%s> <%s> <%s>",
                    cursor.kind().to_uint(),
                    cursor.kind().spelling().to_str(),
                    cursor.spelling().to_str(),
                    cursor.display_name().to_str(),
                    ty.kind().spelling().to_str(),
                    ty.canonical_type().kind().spelling().to_str()));
            }

            let children = cursor.children();
            vec::iter(children, {|cursor| f(*cursor, depth + 1u); });
        }

        f(cursor, 0u);
    }

    #[test]
    fn test() unsafe {
        let index = index(false);
        let tu = index.parse("foo.c", [], [], 0u);

        println("");
        println(#fmt("spelling: %s", tu.spelling().to_str()));

        vec::iter(tu.inclusions(), {|inc|
            println(#fmt("included_file: %s %s",
                inc.included_file.filename().to_str(),
                inc.location.to_str()));
        });

        let cursor = tu.cursor();
        println("");
        print_children(cursor);
    }
}
