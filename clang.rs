use std;
import ctypes::*;
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

    fn clang_createIndex(excludeDeclarationsFromPCH: c_int,
                         displayDiagnostics: c_int) -> CXIndex;
    fn clang_disposeIndex(index: CXIndex);

    fn clang_parseTranslationUnit(
        CIdx: CXIndex,
        source_filename: sbuf,
        command_line_args: *sbuf,
        num_command_line_args: c_int,
        unsaved_files: *CXUnsavedFile,
        num_unsaved_files: unsigned,
        options: unsigned) -> CXTranslationUnit;

    fn clang_disposeTranslationUnit(tu: CXTranslationUnit);

    fn clang_getTranslationUnitSpelling(tu: CXTranslationUnit) -> CXString;

    fn clang_getTranslationUnitCursor(tu: CXTranslationUnit) -> CXCursor;

    fn clang_getCursorKindSpelling(kind: enum) -> CXString;
    fn clang_isDeclaration(kind: enum) -> unsigned;
    fn clang_isReference(kind: enum) -> unsigned;
    fn clang_isExpression(kind: enum) -> unsigned;
    fn clang_isStatement(kind: enum) -> unsigned;
    fn clang_isAttribute(kind: enum) -> unsigned;
    fn clang_isInvalid(kind: enum) -> unsigned;
    fn clang_isTranslationUnit(kind: enum) -> unsigned;
    fn clang_isPreprocessing(kind: enum) -> unsigned;
    fn clang_isUnexposed(kind: enum) -> unsigned;

    fn clang_getTypeKindSpelling(kind: enum) -> CXString;
}

#[link_args = "-L."]
native mod rustclang {
    fn rustclang_getInclusions(tu: clang::CXTranslationUnit,
                               &inclusions: *_file_inclusion,
                               &len: unsigned);

    fn rustclang_getExpansionLocation(location: CXSourceLocation,
                                      &file: clang::CXFile,
                                      &line: unsigned,
                                      &column: unsigned,
                                      &offset: unsigned);

    fn rustclang_visitChildren(parent: CXCursor,
                               &children: *CXCursor,
                               &len: unsigned);

    // Work around bug #1402.
    fn rustclang_getCursorKind(cursor: CXCursor) -> enum;
    fn rustclang_getCursorUSR(cursor: CXCursor, string: CXString);
    fn rustclang_getCursorSpelling(cursor: CXCursor, string: CXString);
    fn rustclang_getCursorDisplayName(cursor: CXCursor, string: CXString);

    fn rustclang_getCursorType(cursor: CXCursor, ty: CXType);
    fn rustclang_getCursorResultType(cursor: CXCursor, ty: CXType);

    fn rustclang_getCanonicalType(in_ty: CXType, ty: CXType);
    fn rustclang_isConstQualified(ty: CXType) -> unsigned;
    fn rustclang_isVolatileQualified(ty: CXType) -> unsigned;
    fn rustclang_isRestrictQualified(ty: CXType) -> unsigned;
    fn rustclang_getPointeeType(in_ty: CXType, out_ty: CXType);
    fn rustclang_getTypeDeclaration(ty: CXType, cursor: CXCursor);
    fn rustclang_getResultType(in_ty: CXType, out_ty: CXType);
    fn rustclang_isPODType(ty: CXType) -> unsigned;
    fn rustclang_getArrayElementType(in_ty: CXType, out_ty: CXType);
    fn rustclang_getArraySize(ty: CXType) -> longlong;
}

// ---------------------------------------------------------------------------

type CXString = {
    data: *void,
    private_flags: unsigned,
};

fn empty_cxstring() -> CXString {
    { data: ptr::null(), private_flags: 0u as unsigned }
}

iface string {
    fn to_str() -> str;
}

// CXString wrapper.
fn new_string(string: CXString) -> string {
    resource string_res(string: CXString) {
        clang::clang_disposeString(string);
    }

    impl of string for string_res {
        fn to_str() -> str unsafe {
            str::from_cstr(clang::clang_getCString(*self))
        }
    }

    string_res(string) as string
}

// ---------------------------------------------------------------------------

type CXSourceLocation = {
    // This should actually be an array of 2 void*, but we can't express that.
    // Hopefully we won't run into alignment issues in the meantime.
    ptr_data0: *void,
    ptr_data1: *void,
    int_data: unsigned,
};

type expansion = {
    file: file,
    line: uint,
    column: uint,
    offset: uint,
};

iface source_location {
    fn expansion() -> expansion;
    fn to_str() -> str;
}

// CXSourceLocation wrapper.
impl of source_location for CXSourceLocation {
    fn expansion() -> expansion unsafe {
        let file = unsafe::reinterpret_cast(0u);
        let line = 0u32;
        let column = 0u32;
        let offset = 0u32;

        rustclang::rustclang_getExpansionLocation(self,
                file,
                line,
                column,
                offset);

        {
            file: file as file,
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

iface file {
    fn filename() -> string;
}

// CXFile wrapper.
impl of file for clang::CXFile {
    fn filename() -> string {
        new_string(clang::clang_getFileName(self))
    }
}

// ---------------------------------------------------------------------------

type CXUnsavedFile = {
    Filename: sbuf,
    Contents: sbuf,
    Length: ulong
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
    {
        included_file: fu.included_file as file,
        location: fu.location as source_location,
        depth: fu.depth
    }
}

// ---------------------------------------------------------------------------

type CXCursor = {
    kind: enum,
    xdata: c_int,
    data0: *void,
    data1: *void,
    data2: *void
};

fn empty_cxcursor() -> CXCursor {
    {
        kind: 0 as enum,
        xdata: 0 as c_int,
        data0: ptr::null(),
        data1: ptr::null(),
        data2: ptr::null(),
    }
}

iface cursor {
    fn kind() -> cursor_kind;
    fn USR() -> string;
    fn spelling() -> string;
    fn display_name() -> string;
    fn children() -> [cursor];
    fn cursor_type() -> cursor_type;
    fn result_type() -> cursor_type;
}

impl of cursor for CXCursor {
    fn kind() -> cursor_kind {
        rustclang::rustclang_getCursorKind(self) as cursor_kind
    }

    fn USR() -> string {
        let string = empty_cxstring();
        rustclang::rustclang_getCursorUSR(self, string);
        new_string(string)
    }

    fn spelling() -> string {
        let string = empty_cxstring();
        rustclang::rustclang_getCursorSpelling(self, string);
        new_string(string)
    }

    fn display_name() -> string {
        let string = empty_cxstring();
        rustclang::rustclang_getCursorDisplayName(self, string);
        new_string(string)
    }

    fn children() -> [cursor] unsafe {
        let len = 0u as unsigned;
        let children = ptr::null::<CXCursor>();
        rustclang::rustclang_visitChildren(self, children, len);
        let len = len as uint;

        let cv : c_vec::t<CXCursor> = c_vec::create(
            unsafe::reinterpret_cast(children),
            len);

        let v = vec::init_fn({|i| c_vec::get(cv, i) as cursor }, len);

        // llvm handles cleaning up the inclusions for us, so we can
        // just let them leak.

        v
    }

    fn cursor_type() -> cursor_type {
        let ty = empty_cxtype();
        rustclang::rustclang_getCursorType(self, ty);
        ty as cursor_type
    }

    fn result_type() -> cursor_type {
        let ty = empty_cxtype();
        rustclang::rustclang_getCursorResultType(self, ty);
        ty as cursor_type
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

iface cursor_kind {
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
}

impl of cursor_kind for enum {
    fn to_uint() -> uint { self as uint }

    fn spelling() -> string {
        new_string(clang::clang_getCursorKindSpelling(self))
    }

    fn is_declaration() -> bool {
        clang::clang_isDeclaration(self) != 0u as unsigned
    }

    fn is_reference() -> bool {
        clang::clang_isReference(self) != 0u as unsigned
    }

    fn is_expression() -> bool {
        clang::clang_isExpression(self) != 0u as unsigned
    }

    fn is_statement() -> bool {
        clang::clang_isStatement(self) != 0u as unsigned
    }

    fn is_attribute() -> bool {
        clang::clang_isAttribute(self) != 0u as unsigned
    }

    fn is_invalid() -> bool {
        clang::clang_isInvalid(self) != 0u as unsigned
    }

    fn is_translation_unit() -> bool {
        clang::clang_isTranslationUnit(self) != 0u as unsigned
    }

    fn is_preprocessing() -> bool {
        clang::clang_isPreprocessing(self) != 0u as unsigned
    }

    fn is_unexposed() -> bool {
        clang::clang_isUnexposed(self) != 0u as unsigned
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

iface cursor_type_kind {
    fn to_uint() -> uint;
    fn spelling() -> string;
}

impl of cursor_type_kind for enum {
    fn to_uint() -> uint {
        self as uint
    }

    fn spelling() -> string {
        new_string(clang::clang_getTypeKindSpelling(self))
    }
}

// ---------------------------------------------------------------------------

type CXType = {
    kind: enum,
    data0: *void,
    data1: *void
};

fn empty_cxtype() -> CXType {
    {
        kind: 0 as enum,
        data0: ptr::null(),
        data1: ptr::null()
    }
}

iface cursor_type {
    fn kind() -> cursor_type_kind;
    fn canonical_type() -> cursor_type;
    fn is_const_qualified() -> bool;
    fn is_volatile_qualified() -> bool;
    fn is_restrict_qualified() -> bool;
    fn pointee_type() -> cursor_type;
    fn type_declaration() -> cursor;
    fn result_type() -> cursor_type;
    fn is_pod_type() -> bool;
    fn array_element_type() -> cursor_type;
    fn array_size() -> u64;
}


impl of cursor_type for CXType {
    fn kind() -> cursor_type_kind {
        self.kind as cursor_type_kind
    }

    fn canonical_type() -> cursor_type {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getCanonicalType(self, out_ty);
        out_ty as cursor_type
    }

    fn is_const_qualified() -> bool {
        rustclang::rustclang_isConstQualified(self) != 0 as unsigned
    }

    fn is_volatile_qualified() -> bool {
        rustclang::rustclang_isVolatileQualified(self) != 0 as unsigned
    }

    fn is_restrict_qualified() -> bool {
        rustclang::rustclang_isRestrictQualified(self) != 0 as unsigned
    }

    fn pointee_type() -> cursor_type {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getPointeeType(self, out_ty);
        out_ty as cursor_type
    }

    fn type_declaration() -> cursor {
        let cursor = empty_cxcursor();
        rustclang::rustclang_getTypeDeclaration(self, cursor);
        cursor as cursor
    }

    fn result_type() -> cursor_type {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getResultType(self, out_ty);
        out_ty as cursor_type
    }

    fn is_pod_type() -> bool {
        rustclang::rustclang_isPODType(self) != 0 as unsigned
    }

    fn array_element_type() -> cursor_type {
        let out_ty = empty_cxtype();
        rustclang::rustclang_getArrayElementType(self, out_ty);
        out_ty as cursor_type
    }

    fn array_size() -> u64 {
        rustclang::rustclang_getArraySize(self) as u64
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

iface translation_unit {
    fn spelling() -> string;
    fn inclusions() -> [file_inclusion];
    fn cursor() -> cursor;
}

// CXTranslationUnit wrapper.
fn new_translation_unit(tu: clang::CXTranslationUnit) -> translation_unit {
    resource translation_unit_res(tu: clang::CXTranslationUnit) {
        clang::clang_disposeTranslationUnit(tu);
    }

    impl of translation_unit for translation_unit_res {
        fn spelling() -> string {
            new_string(clang::clang_getTranslationUnitSpelling(*self))
        }

        fn inclusions() -> [file_inclusion] unsafe {
            // We can't support the native clang_getInclusions, because it
            // needs a callback function which rust doesn't support.
            // Instead we'll make a vector in our stub library and copy the
            // values from it.

            let len = 0u as unsigned;
            let inclusions = ptr::null::<_file_inclusion>();
            rustclang::rustclang_getInclusions(*self, inclusions, len);
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
            clang::clang_getTranslationUnitCursor(*self) as cursor
        }
    }

    translation_unit_res(tu) as translation_unit
}

// ---------------------------------------------------------------------------

iface index {
    fn parse(str, [str], [CXUnsavedFile], uint) -> translation_unit;
}

fn index(excludeDecls: bool) -> index {
    let excludeDeclarationsFromPCH = if excludeDecls { 1 } else { 0 };
    let index = clang::clang_createIndex(
        excludeDeclarationsFromPCH as c_int,
        0 as c_int);

    // CXIndex wrapper.
    resource index_res(index: clang::CXIndex) {
        clang::clang_disposeIndex(index);
    }

    impl of index for index_res {
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
                        *self,
                        str::as_buf(*path, { |buf| buf }),
                        vec::to_ptr(argv),
                        vec::len(argv) as c_int,
                        vec::to_ptr(unsaved_files),
                        vec::len(unsaved_files) as unsigned,
                        options as unsigned)
                };

            new_translation_unit(tu)
        }
    };

    index_res(index) as index
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
            vec::iter(children, {|cursor| f(cursor, depth + 1u); });
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
