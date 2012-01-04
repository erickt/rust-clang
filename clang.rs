use std;
import str::sbuf;
import std::c_vec;
import std::io::{print, println};

// ---------------------------------------------------------------------------

#[link_args = "-L ../rust/build-make/llvm/x86_64-apple-darwin/Release+Asserts/lib"]
native mod clang {
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
}

#[link_args = "-L."]
native mod rustclang {
    fn rustclang_getInclusions(tu: CXTranslationUnit,
                               &inclusions: *_file_inclusion,
                               &len: ctypes::unsigned);

    fn rustclang_getExpansionLocation(location: CXSourceLocation,
                                      &file: CXFile,
                                      &line: ctypes::unsigned,
                                      &column: ctypes::unsigned,
                                      &offset: ctypes::unsigned);

    // Work around bug #1402.
    fn rustclang_getCursorKind(cursor: CXCursor) -> ctypes::enum;
    fn rustclang_getCursorSpelling(cursor: CXCursor) -> CXString;
    fn rustclang_getCursorDisplayName(cursor: CXCursor) -> CXString;

    fn rustclang_visitChildren(parent: CXCursor,
                               &children: *CXCursor,
                               &len: ctypes::unsigned);
}

#[nolink]
native mod libc {
    fn free(ptr: *ctypes::void);
}

// ---------------------------------------------------------------------------

type CXString = {
    data: *ctypes::void,
    private_flags: ctypes::unsigned,
};

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
      line: uint, column: uint, offset: uint };

type source_location = obj {
    fn expansion() -> expansion;
    fn to_str() -> str;
};

// CXSourceLocation wrapper.
obj new_source_location(location: CXSourceLocation) {
    fn expansion() -> expansion unsafe {
        let file : CXFile = unsafe::reinterpret_cast(0u);
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

type CXFile = *ctypes::void;

type file = obj {
    fn filename() -> string;
};

// CXFile wrapper.
obj new_file(file: CXFile) {
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
    included_file: CXFile,
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

// It sure would be nice if rust's objects supported recursive types. In the
// meantime, break up the recursion by inserting a tag into the chain.
tag cursor_tag = cursor;

type cursor = obj {
    fn kind() -> uint;
    fn spelling() -> string;
    fn display_name() -> string;
    fn children() -> [cursor_tag];
};

obj new_cursor(cursor: CXCursor) {
    fn kind() -> uint {
        rustclang::rustclang_getCursorKind(cursor) as uint
    }

    fn spelling() -> string {
        new_string(rustclang::rustclang_getCursorSpelling(cursor))
    }

    fn display_name() -> string {
        new_string(rustclang::rustclang_getCursorDisplayName(cursor))
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
}

// ---------------------------------------------------------------------------

type CXTranslationUnit = *ctypes::void;

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
fn new_translation_unit(tu: CXTranslationUnit) -> translation_unit {
    resource translation_unit_res(tu: CXTranslationUnit) {
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

type CXIndex = *ctypes::void;

type index = obj {
    fn parse(str, [str], [CXUnsavedFile], uint) -> translation_unit;
};

fn index(excludeDecls: bool) -> index {
    let excludeDeclarationsFromPCH = if excludeDecls { 1 } else { 0 };
    let index = clang::clang_createIndex(
        excludeDeclarationsFromPCH as ctypes::c_int,
        0 as ctypes::c_int);

    // CXIndex wrapper.
    resource index_res(index: CXIndex) {
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
            let children = cursor.children();
            vec::iter(children, {|cursor|
                uint::range(0u, depth, { |_i| print(">") });
                println(#fmt("> %s", cursor.display_name().to_str()));
                f(*cursor, depth + 1u);
            });
        }

        println(#fmt("%s", cursor.display_name().to_str()));
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
