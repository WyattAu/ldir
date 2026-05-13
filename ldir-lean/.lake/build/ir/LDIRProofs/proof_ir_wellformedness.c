// Lean compiler output
// Module: LDIRProofs.proof_ir_wellformedness
// Imports: public import Init public import Init
#include <lean/lean.h>
#if defined(__clang__)
#pragma clang diagnostic ignored "-Wunused-parameter"
#pragma clang diagnostic ignored "-Wunused-label"
#elif defined(__GNUC__) && !defined(__CLANG__)
#pragma GCC diagnostic ignored "-Wunused-parameter"
#pragma GCC diagnostic ignored "-Wunused-label"
#pragma GCC diagnostic ignored "-Wunused-but-set-variable"
#endif
#ifdef __cplusplus
extern "C" {
#endif
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorIdx(uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorIdx___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_toCtorIdx(uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_toCtorIdx___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim(lean_object*, lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim___boxed(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 24, .m_capacity = 24, .m_length = 23, .m_data = "LDIR.BlockType.document"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__0_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__1_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 25, .m_capacity = 25, .m_length = 24, .m_data = "LDIR.BlockType.paragraph"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__2_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__3 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__3_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__4_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 23, .m_capacity = 23, .m_length = 22, .m_data = "LDIR.BlockType.heading"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__4 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__4_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__5_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__4_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__5 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__5_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__6_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 20, .m_capacity = 20, .m_length = 19, .m_data = "LDIR.BlockType.list"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__6 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__6_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__7_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__6_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__7 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__7_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__8_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 20, .m_capacity = 20, .m_length = 19, .m_data = "LDIR.BlockType.math"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__8 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__8_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__9_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__8_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__9 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__9_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__10_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 20, .m_capacity = 20, .m_length = 19, .m_data = "LDIR.BlockType.code"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__10 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__10_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__11_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__10_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__11 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__11_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__12_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 26, .m_capacity = 26, .m_length = 25, .m_data = "LDIR.BlockType.blockQuote"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__12 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__12_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__13_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__12_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__13 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__13_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__14_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 29, .m_capacity = 29, .m_length = 28, .m_data = "LDIR.BlockType.thematicBreak"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__14 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__14_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__15_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__14_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__15 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__15_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__16_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 21, .m_capacity = 21, .m_length = 20, .m_data = "LDIR.BlockType.image"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__16 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__16_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__17_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__16_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__17 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__17_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__18_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 21, .m_capacity = 21, .m_length = 20, .m_data = "LDIR.BlockType.table"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__18 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__18_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__19_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__18_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__19 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__19_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__20_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 24, .m_capacity = 24, .m_length = 23, .m_data = "LDIR.BlockType.tableRow"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__20 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__20_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__21_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__20_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__21 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__21_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__22_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 25, .m_capacity = 25, .m_length = 24, .m_data = "LDIR.BlockType.tableCell"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__22 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__22_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__23_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__22_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__23 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__23_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__24_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 24, .m_capacity = 24, .m_length = 23, .m_data = "LDIR.BlockType.footnote"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__24 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__24_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__25_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__24_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__25 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__25_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__26_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 29, .m_capacity = 29, .m_length = 28, .m_data = "LDIR.BlockType.footnoteBlock"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__26 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__26_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__27_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__26_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__27 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__27_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__28_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 22, .m_capacity = 22, .m_length = 21, .m_data = "LDIR.BlockType.figure"};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__28 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__28_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__29_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__28_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__29 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__29_value;
lean_object* lean_nat_to_int(lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31;
lean_object* l_Repr_addAppParen(lean_object*, lean_object*);
uint8_t lean_nat_dec_le(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr(uint8_t, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprBlockType___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprBlockType_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprBlockType___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprBlockType = (const lean_object*)&lp_LDIRProofs_LDIR_instReprBlockType___closed__0_value;
uint8_t lean_nat_dec_eq(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqBlockType_beq(uint8_t, uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqBlockType_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqBlockType___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqBlockType_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqBlockType___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqBlockType___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqBlockType = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqBlockType___closed__0_value;
uint8_t lean_nat_dec_le(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_BlockType_ofNat(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ofNat___boxed(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqBlockType(uint8_t, uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqBlockType___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorIdx(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorIdx___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim___boxed(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim___redArg___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim___redArg___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim___redArg___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim___redArg___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim___redArg___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 26, .m_capacity = 26, .m_length = 25, .m_data = "LDIR.SIROpcode.setContent"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__0_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__1_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 26, .m_capacity = 26, .m_length = 25, .m_data = "LDIR.SIROpcode.applyStyle"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__2_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__3 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__3_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__4_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 26, .m_capacity = 26, .m_length = 25, .m_data = "LDIR.SIROpcode.insertMath"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__4 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__4_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__5_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__4_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__5 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__5_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__6_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 24, .m_capacity = 24, .m_length = 23, .m_data = "LDIR.SIROpcode.linkData"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__6 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__6_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__7_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__6_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__7 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__7_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__8_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 25, .m_capacity = 25, .m_length = 24, .m_data = "LDIR.SIROpcode.pushBlock"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__8 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__8_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__9_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__8_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__9 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__9_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__10_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__9_value),((lean_object*)(((size_t)(1) << 1) | 1))}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__10 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__10_value;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprSIROpcode___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprSIROpcode_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIROpcode___closed__0_value;
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqSIROpcode_beq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqSIROpcode_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqSIROpcode___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqSIROpcode_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqSIROpcode___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqSIROpcode___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqSIROpcode = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqSIROpcode___closed__0_value;
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqSIROpcode_decEq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqSIROpcode_decEq___boxed(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqSIROpcode(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqSIROpcode___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_rootSentinel;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 3, .m_capacity = 3, .m_length = 2, .m_data = "{ "};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__0_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 7, .m_capacity = 7, .m_length = 6, .m_data = "opcode"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__1_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__1_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)(((size_t)(0) << 1) | 1)),((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__2_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__3 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__3_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__4_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 5, .m_capacity = 5, .m_length = 4, .m_data = " := "};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__4 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__4_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__4_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__6_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__3_value),((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__6 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__6_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__7_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__7;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__8_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 2, .m_capacity = 2, .m_length = 1, .m_data = ","};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__8 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__8_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__9_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__8_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__9 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__9_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__10_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 10, .m_capacity = 10, .m_length = 9, .m_data = "entity_id"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__10 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__10_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__11_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__10_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__11 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__11_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__12_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__12;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__13_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 10, .m_capacity = 10, .m_length = 9, .m_data = "parent_id"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__13 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__13_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__14_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__13_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__14 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__14_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__15_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 15, .m_capacity = 15, .m_length = 14, .m_data = "payload_offset"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__15 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__15_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__16_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__15_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__16 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__16_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__17_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__17;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__18_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 3, .m_capacity = 3, .m_length = 2, .m_data = " }"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__18 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__18_value;
lean_object* lean_string_length(lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__19_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__19;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__21_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__21 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__21_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__22_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__18_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__22 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__22_value;
lean_object* l_Nat_reprFast(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprSIRInstruction___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction___closed__0_value;
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqSIRInstruction_beq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqSIRInstruction_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqSIRInstruction___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqSIRInstruction_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqSIRInstruction___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqSIRInstruction___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqSIRInstruction = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqSIRInstruction___closed__0_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 5, .m_capacity = 5, .m_length = 4, .m_data = "data"};
static const lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__0_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__1_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)(((size_t)(0) << 1) | 1)),((lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__1_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__2_value),((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__3 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__3_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__4_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__4;
lean_object* l_String_quote(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprPayloadTable___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprPayloadTable_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable = (const lean_object*)&lp_LDIRProofs_LDIR_instReprPayloadTable___closed__0_value;
uint8_t lean_string_dec_eq(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqPayloadTable_beq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqPayloadTable_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqPayloadTable___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqPayloadTable_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqPayloadTable___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqPayloadTable___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqPayloadTable = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqPayloadTable___closed__0_value;
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00List_foldl___at___00Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0_spec__1_spec__2(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0_spec__1(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0(lean_object*, lean_object*);
static const lean_string_object lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 3, .m_capacity = 3, .m_length = 2, .m_data = "[]"};
static const lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__0 = (const lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__0_value;
static const lean_ctor_object lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__0_value)}};
static const lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__1 = (const lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__1_value;
static const lean_string_object lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 2, .m_capacity = 2, .m_length = 1, .m_data = "["};
static const lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__2 = (const lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__9_value),((lean_object*)(((size_t)(1) << 1) | 1))}};
static const lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__3 = (const lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__3_value;
static const lean_string_object lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__4_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 2, .m_capacity = 2, .m_length = 1, .m_data = "]"};
static const lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__4 = (const lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__4_value;
static lean_once_cell_t lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__5_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__5;
static lean_once_cell_t lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__6_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__6;
static const lean_ctor_object lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__7_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__2_value)}};
static const lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__7 = (const lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__7_value;
static const lean_ctor_object lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__8_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__4_value)}};
static const lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__8 = (const lean_object*)&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__8_value;
LEAN_EXPORT lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg(lean_object*);
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 13, .m_capacity = 13, .m_length = 12, .m_data = "instructions"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__0_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__1_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)(((size_t)(0) << 1) | 1)),((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__1_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__2_value),((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__3 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__3_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4;
static const lean_string_object lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__5_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 8, .m_capacity = 8, .m_length = 7, .m_data = "payload"};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__5 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__5_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__6_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__5_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__6 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__6_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload = (const lean_object*)&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload___closed__0_value;
LEAN_EXPORT uint8_t lp_LDIRProofs_List_beq___at___00LDIR_instBEqSIRDocumentWithPayload_beq_spec__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_beq___at___00LDIR_instBEqSIRDocumentWithPayload_beq_spec__0___boxed(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload_beq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload___closed__0_value;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIR__COMMAND__ARGS;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorIdx___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_toCtorIdx(uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_toCtorIdx___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim(lean_object*, lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim___boxed(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim(lean_object*, uint8_t, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
static const lean_string_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 23, .m_capacity = 23, .m_length = 22, .m_data = "LDIR.GIROpcode.setFont"};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__0_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__1_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 22, .m_capacity = 22, .m_length = 21, .m_data = "LDIR.GIROpcode.moveXY"};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__2_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__3 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__3_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__4_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 24, .m_capacity = 24, .m_length = 23, .m_data = "LDIR.GIROpcode.putGlyph"};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__4 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__4_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__5_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__4_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__5 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__5_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__6_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 24, .m_capacity = 24, .m_length = 23, .m_data = "LDIR.GIROpcode.drawRule"};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__6 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__6_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__7_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__6_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__7 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__7_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__8_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 25, .m_capacity = 25, .m_length = 24, .m_data = "LDIR.GIROpcode.pushStack"};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__8 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__8_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__9_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__8_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__9 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__9_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__10_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 24, .m_capacity = 24, .m_length = 23, .m_data = "LDIR.GIROpcode.popStack"};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__10 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__10_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__11_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__10_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__11 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__11_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__12_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 30, .m_capacity = 30, .m_length = 29, .m_data = "LDIR.GIROpcode.attachMetadata"};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__12 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__12_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__13_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__12_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__13 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__13_value;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr(uint8_t, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprGIROpcode___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprGIROpcode_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode = (const lean_object*)&lp_LDIRProofs_LDIR_instReprGIROpcode___closed__0_value;
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqGIROpcode_beq(uint8_t, uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqGIROpcode_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqGIROpcode___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqGIROpcode_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqGIROpcode___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqGIROpcode___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqGIROpcode = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqGIROpcode___closed__0_value;
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_GIROpcode_ofNat(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ofNat___boxed(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqGIROpcode(uint8_t, uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqGIROpcode___boxed(lean_object*, lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___closed__0;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___boxed(lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_GIRCommand_zeroed___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___boxed, .m_arity = 1, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_GIRCommand_zeroed___closed__0_value;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed(uint8_t);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___boxed(lean_object*);
uint8_t lean_int_dec_eq(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__0(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__0___boxed(lean_object*, lean_object*, lean_object*);
uint8_t l_Nat_decidableForallFin___redArg(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__1(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__1___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqGIRCommand___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__1___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqGIRCommand___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqGIRCommand___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqGIRCommand = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqGIRCommand___closed__0_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instInhabitedGIRCommand;
lean_object* l_List_reverse___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_entityUnique_spec__0(lean_object*, lean_object*);
lean_object* l_instDecidableEqNat___boxed(lean_object*, lean_object*);
uint8_t l_List_nodupDecidable___redArg(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_entityUnique(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_entityUnique___boxed(lean_object*);
lean_object* l_instBEqOfDecidableEq___redArg___lam__0___boxed(lean_object*, lean_object*, lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_parentIdValid___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_parentIdValid___closed__0;
uint8_t l_List_elem___redArg(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_parentIdValid(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_parentIdValid___boxed(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_parentExists___lam__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_parentExists___lam__0___boxed(lean_object*, lean_object*);
uint8_t l_List_all___redArg(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_parentExists(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_parentExists___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_filterTR_loop___at___00LDIR_hasSingleRoot_spec__0(lean_object*, lean_object*);
lean_object* l_List_lengthTR___redArg(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_hasSingleRoot(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_hasSingleRoot___boxed(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclicAux___lam__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclicAux___lam__0___boxed(lean_object*, lean_object*);
lean_object* l_List_find_x3f___redArg(lean_object*, lean_object*);
lean_object* lean_nat_sub(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclicAux(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclicAux___boxed(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclic___lam__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclic___lam__0___boxed(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclic(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclic___boxed(lean_object*);
lean_object* lean_string_length(lean_object*);
uint8_t lean_nat_dec_lt(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_payloadValid___lam__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_payloadValid___lam__0___boxed(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_payloadValid(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_payloadValid___boxed(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_wellFormedSIR(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_wellFormedSIR___boxed(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_wellFormedSIRWithPayload(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_wellFormedSIRWithPayload___boxed(lean_object*);
lean_object* lean_nat_add(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_stackBalancedAux(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_stackBalancedAux___boxed(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_stackBalanced(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_stackBalanced___boxed(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_pageWellFormed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_pageWellFormed___boxed(lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_wellFormedGIR___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_stackBalanced___boxed, .m_arity = 1, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_wellFormedGIR___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_wellFormedGIR___closed__0_value;
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_wellFormedGIR(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_wellFormedGIR___boxed(lean_object*);
static const lean_ctor_object lp_LDIRProofs_LDIR_compileStub___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 1}, .m_objs = {((lean_object*)(((size_t)(0) << 1) | 1)),((lean_object*)(((size_t)(0) << 1) | 1))}};
static const lean_object* lp_LDIRProofs_LDIR_compileStub___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_compileStub___closed__0_value;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileStub(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileStub___boxed(lean_object*);
static lean_once_cell_t lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__0;
static lean_once_cell_t lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__1_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__1;
static lean_once_cell_t lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__2_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__2;
lean_object* l_List_appendTR___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_compileReal_spec__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_compileReal_spec__0___boxed(lean_object*, lean_object*);
uint8_t l_List_isEmpty___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileReal(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileReal___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__4_splitter___redArg(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__4_splitter(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter___redArg(uint8_t, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter___redArg___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter(lean_object*, uint8_t, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter___boxed(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__List_any_match__1_splitter___redArg(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__List_any_match__1_splitter(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter___redArg(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter___redArg___boxed(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__1_splitter___redArg(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__1_splitter(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_payloadValid_match__1_splitter___redArg(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_payloadValid_match__1_splitter(lean_object*, lean_object*, lean_object*, lean_object*);
lean_object* lean_string_utf8_byte_size(lean_object*);
lean_object* l_String_Slice_Pos_nextn(lean_object*, lean_object*, lean_object*);
lean_object* l_String_Slice_toString(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_sirSemanticContent_spec__0(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_sirSemanticContent_spec__0___boxed(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_sirSemanticContent(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__0(lean_object*, lean_object*, lean_object*);
lean_object* l_List_finRange(lean_object*);
static lean_once_cell_t lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0;
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_girSemanticContent_spec__2(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_girSemanticContent(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorIdx(uint8_t x_1) {
_start:
{
switch (x_1) {
case 0:
{
lean_object* x_2; 
x_2 = lean_unsigned_to_nat(0u);
return x_2;
}
case 1:
{
lean_object* x_3; 
x_3 = lean_unsigned_to_nat(1u);
return x_3;
}
case 2:
{
lean_object* x_4; 
x_4 = lean_unsigned_to_nat(2u);
return x_4;
}
case 3:
{
lean_object* x_5; 
x_5 = lean_unsigned_to_nat(3u);
return x_5;
}
case 4:
{
lean_object* x_6; 
x_6 = lean_unsigned_to_nat(4u);
return x_6;
}
case 5:
{
lean_object* x_7; 
x_7 = lean_unsigned_to_nat(5u);
return x_7;
}
case 6:
{
lean_object* x_8; 
x_8 = lean_unsigned_to_nat(6u);
return x_8;
}
case 7:
{
lean_object* x_9; 
x_9 = lean_unsigned_to_nat(7u);
return x_9;
}
case 8:
{
lean_object* x_10; 
x_10 = lean_unsigned_to_nat(8u);
return x_10;
}
case 9:
{
lean_object* x_11; 
x_11 = lean_unsigned_to_nat(9u);
return x_11;
}
case 10:
{
lean_object* x_12; 
x_12 = lean_unsigned_to_nat(10u);
return x_12;
}
case 11:
{
lean_object* x_13; 
x_13 = lean_unsigned_to_nat(11u);
return x_13;
}
case 12:
{
lean_object* x_14; 
x_14 = lean_unsigned_to_nat(12u);
return x_14;
}
case 13:
{
lean_object* x_15; 
x_15 = lean_unsigned_to_nat(13u);
return x_15;
}
default: 
{
lean_object* x_16; 
x_16 = lean_unsigned_to_nat(14u);
return x_16;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorIdx___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lean_unbox(x_1);
x_3 = lp_LDIRProofs_LDIR_BlockType_ctorIdx(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_toCtorIdx(uint8_t x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_ctorIdx(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_toCtorIdx___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lean_unbox(x_1);
x_3 = lp_LDIRProofs_LDIR_BlockType_toCtorIdx(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_ctorElim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim(lean_object* x_1, lean_object* x_2, uint8_t x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
lean_inc(x_5);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ctorElim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
uint8_t x_6; lean_object* x_7; 
x_6 = lean_unbox(x_3);
x_7 = lp_LDIRProofs_LDIR_BlockType_ctorElim(x_1, x_2, x_6, x_4, x_5);
lean_dec(x_5);
lean_dec(x_2);
return x_7;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_document_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_document_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_document_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_paragraph_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_paragraph_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_paragraph_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_heading_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_heading_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_heading_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_list_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_list_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_list_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_math_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_math_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_math_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_code_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_code_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_code_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_blockQuote_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_blockQuote_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_blockQuote_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_thematicBreak_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_image_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_image_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_image_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_table_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_table_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_table_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_tableRow_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableRow_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_tableRow_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_tableCell_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_tableCell_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_tableCell_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_footnote_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnote_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_footnote_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_footnoteBlock_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_BlockType_figure_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_figure_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_BlockType_figure_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(2u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(1u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr(uint8_t x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_10; lean_object* x_17; lean_object* x_24; lean_object* x_31; lean_object* x_38; lean_object* x_45; lean_object* x_52; lean_object* x_59; lean_object* x_66; lean_object* x_73; lean_object* x_80; lean_object* x_87; lean_object* x_94; lean_object* x_101; 
switch (x_1) {
case 0:
{
lean_object* x_108; uint8_t x_109; 
x_108 = lean_unsigned_to_nat(1024u);
x_109 = lean_nat_dec_le(x_108, x_2);
if (x_109 == 0)
{
lean_object* x_110; 
x_110 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_3 = x_110;
goto block_9;
}
else
{
lean_object* x_111; 
x_111 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_3 = x_111;
goto block_9;
}
}
case 1:
{
lean_object* x_112; uint8_t x_113; 
x_112 = lean_unsigned_to_nat(1024u);
x_113 = lean_nat_dec_le(x_112, x_2);
if (x_113 == 0)
{
lean_object* x_114; 
x_114 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_10 = x_114;
goto block_16;
}
else
{
lean_object* x_115; 
x_115 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_10 = x_115;
goto block_16;
}
}
case 2:
{
lean_object* x_116; uint8_t x_117; 
x_116 = lean_unsigned_to_nat(1024u);
x_117 = lean_nat_dec_le(x_116, x_2);
if (x_117 == 0)
{
lean_object* x_118; 
x_118 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_17 = x_118;
goto block_23;
}
else
{
lean_object* x_119; 
x_119 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_17 = x_119;
goto block_23;
}
}
case 3:
{
lean_object* x_120; uint8_t x_121; 
x_120 = lean_unsigned_to_nat(1024u);
x_121 = lean_nat_dec_le(x_120, x_2);
if (x_121 == 0)
{
lean_object* x_122; 
x_122 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_24 = x_122;
goto block_30;
}
else
{
lean_object* x_123; 
x_123 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_24 = x_123;
goto block_30;
}
}
case 4:
{
lean_object* x_124; uint8_t x_125; 
x_124 = lean_unsigned_to_nat(1024u);
x_125 = lean_nat_dec_le(x_124, x_2);
if (x_125 == 0)
{
lean_object* x_126; 
x_126 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_31 = x_126;
goto block_37;
}
else
{
lean_object* x_127; 
x_127 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_31 = x_127;
goto block_37;
}
}
case 5:
{
lean_object* x_128; uint8_t x_129; 
x_128 = lean_unsigned_to_nat(1024u);
x_129 = lean_nat_dec_le(x_128, x_2);
if (x_129 == 0)
{
lean_object* x_130; 
x_130 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_38 = x_130;
goto block_44;
}
else
{
lean_object* x_131; 
x_131 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_38 = x_131;
goto block_44;
}
}
case 6:
{
lean_object* x_132; uint8_t x_133; 
x_132 = lean_unsigned_to_nat(1024u);
x_133 = lean_nat_dec_le(x_132, x_2);
if (x_133 == 0)
{
lean_object* x_134; 
x_134 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_45 = x_134;
goto block_51;
}
else
{
lean_object* x_135; 
x_135 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_45 = x_135;
goto block_51;
}
}
case 7:
{
lean_object* x_136; uint8_t x_137; 
x_136 = lean_unsigned_to_nat(1024u);
x_137 = lean_nat_dec_le(x_136, x_2);
if (x_137 == 0)
{
lean_object* x_138; 
x_138 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_52 = x_138;
goto block_58;
}
else
{
lean_object* x_139; 
x_139 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_52 = x_139;
goto block_58;
}
}
case 8:
{
lean_object* x_140; uint8_t x_141; 
x_140 = lean_unsigned_to_nat(1024u);
x_141 = lean_nat_dec_le(x_140, x_2);
if (x_141 == 0)
{
lean_object* x_142; 
x_142 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_59 = x_142;
goto block_65;
}
else
{
lean_object* x_143; 
x_143 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_59 = x_143;
goto block_65;
}
}
case 9:
{
lean_object* x_144; uint8_t x_145; 
x_144 = lean_unsigned_to_nat(1024u);
x_145 = lean_nat_dec_le(x_144, x_2);
if (x_145 == 0)
{
lean_object* x_146; 
x_146 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_66 = x_146;
goto block_72;
}
else
{
lean_object* x_147; 
x_147 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_66 = x_147;
goto block_72;
}
}
case 10:
{
lean_object* x_148; uint8_t x_149; 
x_148 = lean_unsigned_to_nat(1024u);
x_149 = lean_nat_dec_le(x_148, x_2);
if (x_149 == 0)
{
lean_object* x_150; 
x_150 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_73 = x_150;
goto block_79;
}
else
{
lean_object* x_151; 
x_151 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_73 = x_151;
goto block_79;
}
}
case 11:
{
lean_object* x_152; uint8_t x_153; 
x_152 = lean_unsigned_to_nat(1024u);
x_153 = lean_nat_dec_le(x_152, x_2);
if (x_153 == 0)
{
lean_object* x_154; 
x_154 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_80 = x_154;
goto block_86;
}
else
{
lean_object* x_155; 
x_155 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_80 = x_155;
goto block_86;
}
}
case 12:
{
lean_object* x_156; uint8_t x_157; 
x_156 = lean_unsigned_to_nat(1024u);
x_157 = lean_nat_dec_le(x_156, x_2);
if (x_157 == 0)
{
lean_object* x_158; 
x_158 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_87 = x_158;
goto block_93;
}
else
{
lean_object* x_159; 
x_159 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_87 = x_159;
goto block_93;
}
}
case 13:
{
lean_object* x_160; uint8_t x_161; 
x_160 = lean_unsigned_to_nat(1024u);
x_161 = lean_nat_dec_le(x_160, x_2);
if (x_161 == 0)
{
lean_object* x_162; 
x_162 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_94 = x_162;
goto block_100;
}
else
{
lean_object* x_163; 
x_163 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_94 = x_163;
goto block_100;
}
}
default: 
{
lean_object* x_164; uint8_t x_165; 
x_164 = lean_unsigned_to_nat(1024u);
x_165 = lean_nat_dec_le(x_164, x_2);
if (x_165 == 0)
{
lean_object* x_166; 
x_166 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_101 = x_166;
goto block_107;
}
else
{
lean_object* x_167; 
x_167 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_101 = x_167;
goto block_107;
}
}
}
block_9:
{
lean_object* x_4; lean_object* x_5; uint8_t x_6; lean_object* x_7; lean_object* x_8; 
x_4 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__1));
x_5 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_5, 0, x_3);
lean_ctor_set(x_5, 1, x_4);
x_6 = 0;
x_7 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_7, 0, x_5);
lean_ctor_set_uint8(x_7, sizeof(void*)*1, x_6);
x_8 = l_Repr_addAppParen(x_7, x_2);
return x_8;
}
block_16:
{
lean_object* x_11; lean_object* x_12; uint8_t x_13; lean_object* x_14; lean_object* x_15; 
x_11 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__3));
x_12 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_12, 0, x_10);
lean_ctor_set(x_12, 1, x_11);
x_13 = 0;
x_14 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_14, 0, x_12);
lean_ctor_set_uint8(x_14, sizeof(void*)*1, x_13);
x_15 = l_Repr_addAppParen(x_14, x_2);
return x_15;
}
block_23:
{
lean_object* x_18; lean_object* x_19; uint8_t x_20; lean_object* x_21; lean_object* x_22; 
x_18 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__5));
x_19 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_19, 0, x_17);
lean_ctor_set(x_19, 1, x_18);
x_20 = 0;
x_21 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_21, 0, x_19);
lean_ctor_set_uint8(x_21, sizeof(void*)*1, x_20);
x_22 = l_Repr_addAppParen(x_21, x_2);
return x_22;
}
block_30:
{
lean_object* x_25; lean_object* x_26; uint8_t x_27; lean_object* x_28; lean_object* x_29; 
x_25 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__7));
x_26 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_26, 0, x_24);
lean_ctor_set(x_26, 1, x_25);
x_27 = 0;
x_28 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_28, 0, x_26);
lean_ctor_set_uint8(x_28, sizeof(void*)*1, x_27);
x_29 = l_Repr_addAppParen(x_28, x_2);
return x_29;
}
block_37:
{
lean_object* x_32; lean_object* x_33; uint8_t x_34; lean_object* x_35; lean_object* x_36; 
x_32 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__9));
x_33 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_33, 0, x_31);
lean_ctor_set(x_33, 1, x_32);
x_34 = 0;
x_35 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_35, 0, x_33);
lean_ctor_set_uint8(x_35, sizeof(void*)*1, x_34);
x_36 = l_Repr_addAppParen(x_35, x_2);
return x_36;
}
block_44:
{
lean_object* x_39; lean_object* x_40; uint8_t x_41; lean_object* x_42; lean_object* x_43; 
x_39 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__11));
x_40 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_40, 0, x_38);
lean_ctor_set(x_40, 1, x_39);
x_41 = 0;
x_42 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_42, 0, x_40);
lean_ctor_set_uint8(x_42, sizeof(void*)*1, x_41);
x_43 = l_Repr_addAppParen(x_42, x_2);
return x_43;
}
block_51:
{
lean_object* x_46; lean_object* x_47; uint8_t x_48; lean_object* x_49; lean_object* x_50; 
x_46 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__13));
x_47 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_47, 0, x_45);
lean_ctor_set(x_47, 1, x_46);
x_48 = 0;
x_49 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_49, 0, x_47);
lean_ctor_set_uint8(x_49, sizeof(void*)*1, x_48);
x_50 = l_Repr_addAppParen(x_49, x_2);
return x_50;
}
block_58:
{
lean_object* x_53; lean_object* x_54; uint8_t x_55; lean_object* x_56; lean_object* x_57; 
x_53 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__15));
x_54 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_54, 0, x_52);
lean_ctor_set(x_54, 1, x_53);
x_55 = 0;
x_56 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_56, 0, x_54);
lean_ctor_set_uint8(x_56, sizeof(void*)*1, x_55);
x_57 = l_Repr_addAppParen(x_56, x_2);
return x_57;
}
block_65:
{
lean_object* x_60; lean_object* x_61; uint8_t x_62; lean_object* x_63; lean_object* x_64; 
x_60 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__17));
x_61 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_61, 0, x_59);
lean_ctor_set(x_61, 1, x_60);
x_62 = 0;
x_63 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_63, 0, x_61);
lean_ctor_set_uint8(x_63, sizeof(void*)*1, x_62);
x_64 = l_Repr_addAppParen(x_63, x_2);
return x_64;
}
block_72:
{
lean_object* x_67; lean_object* x_68; uint8_t x_69; lean_object* x_70; lean_object* x_71; 
x_67 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__19));
x_68 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_68, 0, x_66);
lean_ctor_set(x_68, 1, x_67);
x_69 = 0;
x_70 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_70, 0, x_68);
lean_ctor_set_uint8(x_70, sizeof(void*)*1, x_69);
x_71 = l_Repr_addAppParen(x_70, x_2);
return x_71;
}
block_79:
{
lean_object* x_74; lean_object* x_75; uint8_t x_76; lean_object* x_77; lean_object* x_78; 
x_74 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__21));
x_75 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_75, 0, x_73);
lean_ctor_set(x_75, 1, x_74);
x_76 = 0;
x_77 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_77, 0, x_75);
lean_ctor_set_uint8(x_77, sizeof(void*)*1, x_76);
x_78 = l_Repr_addAppParen(x_77, x_2);
return x_78;
}
block_86:
{
lean_object* x_81; lean_object* x_82; uint8_t x_83; lean_object* x_84; lean_object* x_85; 
x_81 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__23));
x_82 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_82, 0, x_80);
lean_ctor_set(x_82, 1, x_81);
x_83 = 0;
x_84 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_84, 0, x_82);
lean_ctor_set_uint8(x_84, sizeof(void*)*1, x_83);
x_85 = l_Repr_addAppParen(x_84, x_2);
return x_85;
}
block_93:
{
lean_object* x_88; lean_object* x_89; uint8_t x_90; lean_object* x_91; lean_object* x_92; 
x_88 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__25));
x_89 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_89, 0, x_87);
lean_ctor_set(x_89, 1, x_88);
x_90 = 0;
x_91 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_91, 0, x_89);
lean_ctor_set_uint8(x_91, sizeof(void*)*1, x_90);
x_92 = l_Repr_addAppParen(x_91, x_2);
return x_92;
}
block_100:
{
lean_object* x_95; lean_object* x_96; uint8_t x_97; lean_object* x_98; lean_object* x_99; 
x_95 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__27));
x_96 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_96, 0, x_94);
lean_ctor_set(x_96, 1, x_95);
x_97 = 0;
x_98 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_98, 0, x_96);
lean_ctor_set_uint8(x_98, sizeof(void*)*1, x_97);
x_99 = l_Repr_addAppParen(x_98, x_2);
return x_99;
}
block_107:
{
lean_object* x_102; lean_object* x_103; uint8_t x_104; lean_object* x_105; lean_object* x_106; 
x_102 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__29));
x_103 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_103, 0, x_101);
lean_ctor_set(x_103, 1, x_102);
x_104 = 0;
x_105 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_105, 0, x_103);
lean_ctor_set_uint8(x_105, sizeof(void*)*1, x_104);
x_106 = l_Repr_addAppParen(x_105, x_2);
return x_106;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprBlockType_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lean_unbox(x_1);
x_4 = lp_LDIRProofs_LDIR_instReprBlockType_repr(x_3, x_2);
lean_dec(x_2);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqBlockType_beq(uint8_t x_1, uint8_t x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lp_LDIRProofs_LDIR_BlockType_ctorIdx(x_1);
x_4 = lp_LDIRProofs_LDIR_BlockType_ctorIdx(x_2);
x_5 = lean_nat_dec_eq(x_3, x_4);
lean_dec(x_4);
lean_dec(x_3);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqBlockType_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; uint8_t x_4; uint8_t x_5; lean_object* x_6; 
x_3 = lean_unbox(x_1);
x_4 = lean_unbox(x_2);
x_5 = lp_LDIRProofs_LDIR_instBEqBlockType_beq(x_3, x_4);
x_6 = lean_box(x_5);
return x_6;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_BlockType_ofNat(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
x_2 = lean_unsigned_to_nat(6u);
x_3 = lean_nat_dec_le(x_1, x_2);
if (x_3 == 0)
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(10u);
x_5 = lean_nat_dec_le(x_1, x_4);
if (x_5 == 0)
{
lean_object* x_6; uint8_t x_7; 
x_6 = lean_unsigned_to_nat(12u);
x_7 = lean_nat_dec_le(x_1, x_6);
if (x_7 == 0)
{
lean_object* x_8; uint8_t x_9; 
x_8 = lean_unsigned_to_nat(13u);
x_9 = lean_nat_dec_le(x_1, x_8);
if (x_9 == 0)
{
uint8_t x_10; 
x_10 = 14;
return x_10;
}
else
{
uint8_t x_11; 
x_11 = 13;
return x_11;
}
}
else
{
lean_object* x_12; uint8_t x_13; 
x_12 = lean_unsigned_to_nat(11u);
x_13 = lean_nat_dec_le(x_1, x_12);
if (x_13 == 0)
{
uint8_t x_14; 
x_14 = 12;
return x_14;
}
else
{
uint8_t x_15; 
x_15 = 11;
return x_15;
}
}
}
else
{
lean_object* x_16; uint8_t x_17; 
x_16 = lean_unsigned_to_nat(8u);
x_17 = lean_nat_dec_le(x_1, x_16);
if (x_17 == 0)
{
lean_object* x_18; uint8_t x_19; 
x_18 = lean_unsigned_to_nat(9u);
x_19 = lean_nat_dec_le(x_1, x_18);
if (x_19 == 0)
{
uint8_t x_20; 
x_20 = 10;
return x_20;
}
else
{
uint8_t x_21; 
x_21 = 9;
return x_21;
}
}
else
{
lean_object* x_22; uint8_t x_23; 
x_22 = lean_unsigned_to_nat(7u);
x_23 = lean_nat_dec_le(x_1, x_22);
if (x_23 == 0)
{
uint8_t x_24; 
x_24 = 8;
return x_24;
}
else
{
uint8_t x_25; 
x_25 = 7;
return x_25;
}
}
}
}
else
{
lean_object* x_26; uint8_t x_27; 
x_26 = lean_unsigned_to_nat(2u);
x_27 = lean_nat_dec_le(x_1, x_26);
if (x_27 == 0)
{
lean_object* x_28; uint8_t x_29; 
x_28 = lean_unsigned_to_nat(4u);
x_29 = lean_nat_dec_le(x_1, x_28);
if (x_29 == 0)
{
lean_object* x_30; uint8_t x_31; 
x_30 = lean_unsigned_to_nat(5u);
x_31 = lean_nat_dec_le(x_1, x_30);
if (x_31 == 0)
{
uint8_t x_32; 
x_32 = 6;
return x_32;
}
else
{
uint8_t x_33; 
x_33 = 5;
return x_33;
}
}
else
{
lean_object* x_34; uint8_t x_35; 
x_34 = lean_unsigned_to_nat(3u);
x_35 = lean_nat_dec_le(x_1, x_34);
if (x_35 == 0)
{
uint8_t x_36; 
x_36 = 4;
return x_36;
}
else
{
uint8_t x_37; 
x_37 = 3;
return x_37;
}
}
}
else
{
lean_object* x_38; uint8_t x_39; 
x_38 = lean_unsigned_to_nat(0u);
x_39 = lean_nat_dec_le(x_1, x_38);
if (x_39 == 0)
{
lean_object* x_40; uint8_t x_41; 
x_40 = lean_unsigned_to_nat(1u);
x_41 = lean_nat_dec_le(x_1, x_40);
if (x_41 == 0)
{
uint8_t x_42; 
x_42 = 2;
return x_42;
}
else
{
uint8_t x_43; 
x_43 = 1;
return x_43;
}
}
else
{
uint8_t x_44; 
x_44 = 0;
return x_44;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_BlockType_ofNat___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_BlockType_ofNat(x_1);
lean_dec(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqBlockType(uint8_t x_1, uint8_t x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lp_LDIRProofs_LDIR_BlockType_ctorIdx(x_1);
x_4 = lp_LDIRProofs_LDIR_BlockType_ctorIdx(x_2);
x_5 = lean_nat_dec_eq(x_3, x_4);
lean_dec(x_4);
lean_dec(x_3);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqBlockType___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; uint8_t x_4; uint8_t x_5; lean_object* x_6; 
x_3 = lean_unbox(x_1);
x_4 = lean_unbox(x_2);
x_5 = lp_LDIRProofs_LDIR_instDecidableEqBlockType(x_3, x_4);
x_6 = lean_box(x_5);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorIdx(lean_object* x_1) {
_start:
{
switch (lean_obj_tag(x_1)) {
case 0:
{
lean_object* x_2; 
x_2 = lean_unsigned_to_nat(0u);
return x_2;
}
case 1:
{
lean_object* x_3; 
x_3 = lean_unsigned_to_nat(1u);
return x_3;
}
case 2:
{
lean_object* x_4; 
x_4 = lean_unsigned_to_nat(2u);
return x_4;
}
case 3:
{
lean_object* x_5; 
x_5 = lean_unsigned_to_nat(3u);
return x_5;
}
default: 
{
lean_object* x_6; 
x_6 = lean_unsigned_to_nat(4u);
return x_6;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorIdx___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_SIROpcode_ctorIdx(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
uint8_t x_3; lean_object* x_4; lean_object* x_5; 
x_3 = lean_ctor_get_uint8(x_1, 0);
x_4 = lean_box(x_3);
x_5 = lean_apply_1(x_2, x_4);
return x_5;
}
else
{
return x_2;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_1, x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
lean_object* x_6; 
x_6 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_3, x_5);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_ctorElim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
lean_object* x_6; 
x_6 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim(x_1, x_2, x_3, x_4, x_5);
lean_dec(x_3);
lean_dec(x_2);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim___redArg___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim___redArg(x_1, x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_pushBlock_elim(x_1, x_2, x_3, x_4);
lean_dec(x_2);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim___redArg___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_setContent_elim___redArg(x_1, x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_setContent_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_setContent_elim(x_1, x_2, x_3, x_4);
lean_dec(x_2);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim___redArg___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim___redArg(x_1, x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_applyStyle_elim(x_1, x_2, x_3, x_4);
lean_dec(x_2);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim___redArg___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim___redArg(x_1, x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_insertMath_elim(x_1, x_2, x_3, x_4);
lean_dec(x_2);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim___redArg___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_SIROpcode_linkData_elim___redArg(x_1, x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_SIROpcode_linkData_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_SIROpcode_linkData_elim(x_1, x_2, x_3, x_4);
lean_dec(x_2);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_10; lean_object* x_17; lean_object* x_24; 
switch (lean_obj_tag(x_1)) {
case 0:
{
uint8_t x_31; lean_object* x_32; lean_object* x_42; uint8_t x_43; 
x_31 = lean_ctor_get_uint8(x_1, 0);
x_42 = lean_unsigned_to_nat(1024u);
x_43 = lean_nat_dec_le(x_42, x_2);
if (x_43 == 0)
{
lean_object* x_44; 
x_44 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_32 = x_44;
goto block_41;
}
else
{
lean_object* x_45; 
x_45 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_32 = x_45;
goto block_41;
}
block_41:
{
lean_object* x_33; lean_object* x_34; lean_object* x_35; lean_object* x_36; lean_object* x_37; uint8_t x_38; lean_object* x_39; lean_object* x_40; 
x_33 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__10));
x_34 = lean_unsigned_to_nat(1024u);
x_35 = lp_LDIRProofs_LDIR_instReprBlockType_repr(x_31, x_34);
x_36 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_36, 0, x_33);
lean_ctor_set(x_36, 1, x_35);
x_37 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_37, 0, x_32);
lean_ctor_set(x_37, 1, x_36);
x_38 = 0;
x_39 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_39, 0, x_37);
lean_ctor_set_uint8(x_39, sizeof(void*)*1, x_38);
x_40 = l_Repr_addAppParen(x_39, x_2);
return x_40;
}
}
case 1:
{
lean_object* x_46; uint8_t x_47; 
x_46 = lean_unsigned_to_nat(1024u);
x_47 = lean_nat_dec_le(x_46, x_2);
if (x_47 == 0)
{
lean_object* x_48; 
x_48 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_3 = x_48;
goto block_9;
}
else
{
lean_object* x_49; 
x_49 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_3 = x_49;
goto block_9;
}
}
case 2:
{
lean_object* x_50; uint8_t x_51; 
x_50 = lean_unsigned_to_nat(1024u);
x_51 = lean_nat_dec_le(x_50, x_2);
if (x_51 == 0)
{
lean_object* x_52; 
x_52 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_10 = x_52;
goto block_16;
}
else
{
lean_object* x_53; 
x_53 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_10 = x_53;
goto block_16;
}
}
case 3:
{
lean_object* x_54; uint8_t x_55; 
x_54 = lean_unsigned_to_nat(1024u);
x_55 = lean_nat_dec_le(x_54, x_2);
if (x_55 == 0)
{
lean_object* x_56; 
x_56 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_17 = x_56;
goto block_23;
}
else
{
lean_object* x_57; 
x_57 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_17 = x_57;
goto block_23;
}
}
default: 
{
lean_object* x_58; uint8_t x_59; 
x_58 = lean_unsigned_to_nat(1024u);
x_59 = lean_nat_dec_le(x_58, x_2);
if (x_59 == 0)
{
lean_object* x_60; 
x_60 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_24 = x_60;
goto block_30;
}
else
{
lean_object* x_61; 
x_61 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_24 = x_61;
goto block_30;
}
}
}
block_9:
{
lean_object* x_4; lean_object* x_5; uint8_t x_6; lean_object* x_7; lean_object* x_8; 
x_4 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__1));
x_5 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_5, 0, x_3);
lean_ctor_set(x_5, 1, x_4);
x_6 = 0;
x_7 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_7, 0, x_5);
lean_ctor_set_uint8(x_7, sizeof(void*)*1, x_6);
x_8 = l_Repr_addAppParen(x_7, x_2);
return x_8;
}
block_16:
{
lean_object* x_11; lean_object* x_12; uint8_t x_13; lean_object* x_14; lean_object* x_15; 
x_11 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__3));
x_12 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_12, 0, x_10);
lean_ctor_set(x_12, 1, x_11);
x_13 = 0;
x_14 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_14, 0, x_12);
lean_ctor_set_uint8(x_14, sizeof(void*)*1, x_13);
x_15 = l_Repr_addAppParen(x_14, x_2);
return x_15;
}
block_23:
{
lean_object* x_18; lean_object* x_19; uint8_t x_20; lean_object* x_21; lean_object* x_22; 
x_18 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__5));
x_19 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_19, 0, x_17);
lean_ctor_set(x_19, 1, x_18);
x_20 = 0;
x_21 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_21, 0, x_19);
lean_ctor_set_uint8(x_21, sizeof(void*)*1, x_20);
x_22 = l_Repr_addAppParen(x_21, x_2);
return x_22;
}
block_30:
{
lean_object* x_25; lean_object* x_26; uint8_t x_27; lean_object* x_28; lean_object* x_29; 
x_25 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIROpcode_repr___closed__7));
x_26 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_26, 0, x_24);
lean_ctor_set(x_26, 1, x_25);
x_27 = 0;
x_28 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_28, 0, x_26);
lean_ctor_set_uint8(x_28, sizeof(void*)*1, x_27);
x_29 = l_Repr_addAppParen(x_28, x_2);
return x_29;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIROpcode_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprSIROpcode_repr(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqSIROpcode_beq(lean_object* x_1, lean_object* x_2) {
_start:
{
switch (lean_obj_tag(x_1)) {
case 0:
{
if (lean_obj_tag(x_2) == 0)
{
uint8_t x_3; uint8_t x_4; uint8_t x_5; 
x_3 = lean_ctor_get_uint8(x_1, 0);
x_4 = lean_ctor_get_uint8(x_2, 0);
x_5 = lp_LDIRProofs_LDIR_instBEqBlockType_beq(x_3, x_4);
return x_5;
}
else
{
uint8_t x_6; 
x_6 = 0;
return x_6;
}
}
case 1:
{
if (lean_obj_tag(x_2) == 1)
{
uint8_t x_7; 
x_7 = 1;
return x_7;
}
else
{
uint8_t x_8; 
x_8 = 0;
return x_8;
}
}
case 2:
{
if (lean_obj_tag(x_2) == 2)
{
uint8_t x_9; 
x_9 = 1;
return x_9;
}
else
{
uint8_t x_10; 
x_10 = 0;
return x_10;
}
}
case 3:
{
if (lean_obj_tag(x_2) == 3)
{
uint8_t x_11; 
x_11 = 1;
return x_11;
}
else
{
uint8_t x_12; 
x_12 = 0;
return x_12;
}
}
default: 
{
if (lean_obj_tag(x_2) == 4)
{
uint8_t x_13; 
x_13 = 1;
return x_13;
}
else
{
uint8_t x_14; 
x_14 = 0;
return x_14;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqSIROpcode_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instBEqSIROpcode_beq(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqSIROpcode_decEq(lean_object* x_1, lean_object* x_2) {
_start:
{
switch (lean_obj_tag(x_1)) {
case 0:
{
uint8_t x_3; uint8_t x_4; 
x_3 = lean_ctor_get_uint8(x_1, 0);
x_4 = 0;
if (lean_obj_tag(x_2) == 0)
{
uint8_t x_5; uint8_t x_6; 
x_5 = lean_ctor_get_uint8(x_2, 0);
x_6 = lp_LDIRProofs_LDIR_instDecidableEqBlockType(x_3, x_5);
if (x_6 == 0)
{
return x_4;
}
else
{
return x_6;
}
}
else
{
return x_4;
}
}
case 1:
{
switch (lean_obj_tag(x_2)) {
case 0:
{
uint8_t x_7; 
x_7 = 0;
return x_7;
}
case 1:
{
uint8_t x_8; 
x_8 = 1;
return x_8;
}
default: 
{
uint8_t x_9; 
x_9 = 0;
return x_9;
}
}
}
case 2:
{
switch (lean_obj_tag(x_2)) {
case 0:
{
uint8_t x_10; 
x_10 = 0;
return x_10;
}
case 2:
{
uint8_t x_11; 
x_11 = 1;
return x_11;
}
default: 
{
uint8_t x_12; 
x_12 = 0;
return x_12;
}
}
}
case 3:
{
switch (lean_obj_tag(x_2)) {
case 0:
{
uint8_t x_13; 
x_13 = 0;
return x_13;
}
case 3:
{
uint8_t x_14; 
x_14 = 1;
return x_14;
}
default: 
{
uint8_t x_15; 
x_15 = 0;
return x_15;
}
}
}
default: 
{
switch (lean_obj_tag(x_2)) {
case 0:
{
uint8_t x_16; 
x_16 = 0;
return x_16;
}
case 4:
{
uint8_t x_17; 
x_17 = 1;
return x_17;
}
default: 
{
uint8_t x_18; 
x_18 = 0;
return x_18;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqSIROpcode_decEq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instDecidableEqSIROpcode_decEq(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqSIROpcode(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; 
x_3 = lp_LDIRProofs_LDIR_instDecidableEqSIROpcode_decEq(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqSIROpcode___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instDecidableEqSIROpcode(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_rootSentinel(void) {
_start:
{
lean_object* x_1; 
x_1 = lean_unsigned_to_nat(4294967295u);
return x_1;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__7(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(10u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__12(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(13u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__17(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(18u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__19(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__0));
x_2 = lean_string_length(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__19, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__19_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__19);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; uint8_t x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; lean_object* x_23; lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; lean_object* x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; lean_object* x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; lean_object* x_42; lean_object* x_43; lean_object* x_44; lean_object* x_45; lean_object* x_46; lean_object* x_47; lean_object* x_48; lean_object* x_49; lean_object* x_50; lean_object* x_51; lean_object* x_52; lean_object* x_53; lean_object* x_54; lean_object* x_55; 
x_2 = lean_ctor_get(x_1, 0);
lean_inc(x_2);
x_3 = lean_ctor_get(x_1, 1);
lean_inc(x_3);
x_4 = lean_ctor_get(x_1, 2);
lean_inc(x_4);
x_5 = lean_ctor_get(x_1, 3);
lean_inc(x_5);
lean_dec_ref(x_1);
x_6 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5));
x_7 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__6));
x_8 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__7, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__7_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__7);
x_9 = lean_unsigned_to_nat(0u);
x_10 = lp_LDIRProofs_LDIR_instReprSIROpcode_repr(x_2, x_9);
lean_dec(x_2);
x_11 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_11, 0, x_8);
lean_ctor_set(x_11, 1, x_10);
x_12 = 0;
x_13 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_13, 0, x_11);
lean_ctor_set_uint8(x_13, sizeof(void*)*1, x_12);
x_14 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_14, 0, x_7);
lean_ctor_set(x_14, 1, x_13);
x_15 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__9));
x_16 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_16, 0, x_14);
lean_ctor_set(x_16, 1, x_15);
x_17 = lean_box(1);
x_18 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_18, 0, x_16);
lean_ctor_set(x_18, 1, x_17);
x_19 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__11));
x_20 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_20, 0, x_18);
lean_ctor_set(x_20, 1, x_19);
x_21 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_6);
x_22 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__12, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__12_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__12);
x_23 = l_Nat_reprFast(x_3);
x_24 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_24, 0, x_23);
x_25 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_25, 0, x_22);
lean_ctor_set(x_25, 1, x_24);
x_26 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_26, 0, x_25);
lean_ctor_set_uint8(x_26, sizeof(void*)*1, x_12);
x_27 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_27, 0, x_21);
lean_ctor_set(x_27, 1, x_26);
x_28 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_28, 0, x_27);
lean_ctor_set(x_28, 1, x_15);
x_29 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_29, 0, x_28);
lean_ctor_set(x_29, 1, x_17);
x_30 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__14));
x_31 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_31, 0, x_29);
lean_ctor_set(x_31, 1, x_30);
x_32 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_32, 0, x_31);
lean_ctor_set(x_32, 1, x_6);
x_33 = l_Nat_reprFast(x_4);
x_34 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_34, 0, x_33);
x_35 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_35, 0, x_22);
lean_ctor_set(x_35, 1, x_34);
x_36 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_36, 0, x_35);
lean_ctor_set_uint8(x_36, sizeof(void*)*1, x_12);
x_37 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_37, 0, x_32);
lean_ctor_set(x_37, 1, x_36);
x_38 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_38, 0, x_37);
lean_ctor_set(x_38, 1, x_15);
x_39 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_39, 0, x_38);
lean_ctor_set(x_39, 1, x_17);
x_40 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__16));
x_41 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_41, 0, x_39);
lean_ctor_set(x_41, 1, x_40);
x_42 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_42, 0, x_41);
lean_ctor_set(x_42, 1, x_6);
x_43 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__17, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__17_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__17);
x_44 = l_Nat_reprFast(x_5);
x_45 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_45, 0, x_44);
x_46 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_46, 0, x_43);
lean_ctor_set(x_46, 1, x_45);
x_47 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_47, 0, x_46);
lean_ctor_set_uint8(x_47, sizeof(void*)*1, x_12);
x_48 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_48, 0, x_42);
lean_ctor_set(x_48, 1, x_47);
x_49 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20);
x_50 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__21));
x_51 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_51, 0, x_50);
lean_ctor_set(x_51, 1, x_48);
x_52 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__22));
x_53 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_53, 0, x_51);
lean_ctor_set(x_53, 1, x_52);
x_54 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_54, 0, x_49);
lean_ctor_set(x_54, 1, x_53);
x_55 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_55, 0, x_54);
lean_ctor_set_uint8(x_55, sizeof(void*)*1, x_12);
return x_55;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqSIRInstruction_beq(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; uint8_t x_11; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_1, 1);
x_5 = lean_ctor_get(x_1, 2);
x_6 = lean_ctor_get(x_1, 3);
x_7 = lean_ctor_get(x_2, 0);
x_8 = lean_ctor_get(x_2, 1);
x_9 = lean_ctor_get(x_2, 2);
x_10 = lean_ctor_get(x_2, 3);
x_11 = lp_LDIRProofs_LDIR_instBEqSIROpcode_beq(x_3, x_7);
if (x_11 == 0)
{
return x_11;
}
else
{
uint8_t x_12; 
x_12 = lean_nat_dec_eq(x_4, x_8);
if (x_12 == 0)
{
return x_12;
}
else
{
uint8_t x_13; 
x_13 = lean_nat_dec_eq(x_5, x_9);
if (x_13 == 0)
{
return x_13;
}
else
{
uint8_t x_14; 
x_14 = lean_nat_dec_eq(x_6, x_10);
return x_14;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqSIRInstruction_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instBEqSIRInstruction_beq(x_1, x_2);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__4(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(8u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; uint8_t x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; lean_object* x_16; 
x_2 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__3));
x_3 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__4, &lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__4_once, _init_lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg___closed__4);
x_4 = l_String_quote(x_1);
x_5 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_5, 0, x_4);
x_6 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_6, 0, x_3);
lean_ctor_set(x_6, 1, x_5);
x_7 = 0;
x_8 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_8, 0, x_6);
lean_ctor_set_uint8(x_8, sizeof(void*)*1, x_7);
x_9 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_9, 0, x_2);
lean_ctor_set(x_9, 1, x_8);
x_10 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20);
x_11 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__21));
x_12 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_12, 0, x_11);
lean_ctor_set(x_12, 1, x_9);
x_13 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__22));
x_14 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_14, 0, x_12);
lean_ctor_set(x_14, 1, x_13);
x_15 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_15, 0, x_10);
lean_ctor_set(x_15, 1, x_14);
x_16 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_16, 0, x_15);
lean_ctor_set_uint8(x_16, sizeof(void*)*1, x_7);
return x_16;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprPayloadTable_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprPayloadTable_repr(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqPayloadTable_beq(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; 
x_3 = lean_string_dec_eq(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqPayloadTable_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instBEqPayloadTable_beq(x_1, x_2);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00List_foldl___at___00Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0_spec__1_spec__2(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_3) == 0)
{
lean_dec(x_1);
return x_2;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_3);
if (x_4 == 0)
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; 
x_5 = lean_ctor_get(x_3, 0);
x_6 = lean_ctor_get(x_3, 1);
lean_inc(x_1);
lean_ctor_set_tag(x_3, 5);
lean_ctor_set(x_3, 1, x_1);
lean_ctor_set(x_3, 0, x_2);
x_7 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(x_5);
x_8 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_8, 0, x_3);
lean_ctor_set(x_8, 1, x_7);
x_2 = x_8;
x_3 = x_6;
goto _start;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_3, 0);
x_11 = lean_ctor_get(x_3, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_3);
lean_inc(x_1);
x_12 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_12, 0, x_2);
lean_ctor_set(x_12, 1, x_1);
x_13 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(x_10);
x_14 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_14, 0, x_12);
lean_ctor_set(x_14, 1, x_13);
x_2 = x_14;
x_3 = x_11;
goto _start;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0_spec__1(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_3) == 0)
{
lean_dec(x_1);
return x_2;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_3);
if (x_4 == 0)
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_5 = lean_ctor_get(x_3, 0);
x_6 = lean_ctor_get(x_3, 1);
lean_inc(x_1);
lean_ctor_set_tag(x_3, 5);
lean_ctor_set(x_3, 1, x_1);
lean_ctor_set(x_3, 0, x_2);
x_7 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(x_5);
x_8 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_8, 0, x_3);
lean_ctor_set(x_8, 1, x_7);
x_9 = lp_LDIRProofs_List_foldl___at___00List_foldl___at___00Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0_spec__1_spec__2(x_1, x_8, x_6);
return x_9;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_10 = lean_ctor_get(x_3, 0);
x_11 = lean_ctor_get(x_3, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_3);
lean_inc(x_1);
x_12 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_12, 0, x_2);
lean_ctor_set(x_12, 1, x_1);
x_13 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(x_10);
x_14 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_14, 0, x_12);
lean_ctor_set(x_14, 1, x_13);
x_15 = lp_LDIRProofs_List_foldl___at___00List_foldl___at___00Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0_spec__1_spec__2(x_1, x_14, x_11);
return x_15;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_3; 
lean_dec(x_2);
x_3 = lean_box(0);
return x_3;
}
else
{
lean_object* x_4; 
x_4 = lean_ctor_get(x_1, 1);
if (lean_obj_tag(x_4) == 0)
{
lean_object* x_5; lean_object* x_6; 
lean_dec(x_2);
x_5 = lean_ctor_get(x_1, 0);
lean_inc(x_5);
lean_dec_ref(x_1);
x_6 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(x_5);
return x_6;
}
else
{
lean_object* x_7; lean_object* x_8; lean_object* x_9; 
lean_inc(x_4);
x_7 = lean_ctor_get(x_1, 0);
lean_inc(x_7);
lean_dec_ref(x_1);
x_8 = lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg(x_7);
x_9 = lp_LDIRProofs_List_foldl___at___00Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0_spec__1(x_2, x_8, x_4);
return x_9;
}
}
}
}
static lean_object* _init_lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__5(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = ((lean_object*)(lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__2));
x_2 = lean_string_length(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__6(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_obj_once(&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__5, &lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__5_once, _init_lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__5);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg(lean_object* x_1) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_2; 
x_2 = ((lean_object*)(lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__1));
return x_2;
}
else
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; uint8_t x_11; lean_object* x_12; 
x_3 = ((lean_object*)(lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__3));
x_4 = lp_LDIRProofs_Std_Format_joinSep___at___00List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0_spec__0(x_1, x_3);
x_5 = lean_obj_once(&lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__6, &lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__6_once, _init_lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__6);
x_6 = ((lean_object*)(lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__7));
x_7 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_4);
x_8 = ((lean_object*)(lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg___closed__8));
x_9 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_9, 0, x_7);
lean_ctor_set(x_9, 1, x_8);
x_10 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_10, 0, x_5);
lean_ctor_set(x_10, 1, x_9);
x_11 = 0;
x_12 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_12, 0, x_10);
lean_ctor_set_uint8(x_12, sizeof(void*)*1, x_11);
return x_12;
}
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(16u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(11u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg(lean_object* x_1) {
_start:
{
uint8_t x_2; 
x_2 = !lean_is_exclusive(x_1);
if (x_2 == 0)
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; uint8_t x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; lean_object* x_23; lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; lean_object* x_30; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_1, 1);
x_5 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5));
x_6 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__3));
x_7 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4, &lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4_once, _init_lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4);
x_8 = lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg(x_3);
lean_ctor_set_tag(x_1, 4);
lean_ctor_set(x_1, 1, x_8);
lean_ctor_set(x_1, 0, x_7);
x_9 = 0;
x_10 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_10, 0, x_1);
lean_ctor_set_uint8(x_10, sizeof(void*)*1, x_9);
x_11 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_11, 0, x_6);
lean_ctor_set(x_11, 1, x_10);
x_12 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__9));
x_13 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_13, 0, x_11);
lean_ctor_set(x_13, 1, x_12);
x_14 = lean_box(1);
x_15 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_15, 0, x_13);
lean_ctor_set(x_15, 1, x_14);
x_16 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__6));
x_17 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_17, 0, x_15);
lean_ctor_set(x_17, 1, x_16);
x_18 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_18, 0, x_17);
lean_ctor_set(x_18, 1, x_5);
x_19 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7, &lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7_once, _init_lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7);
x_20 = lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg(x_4);
x_21 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_21, 0, x_19);
lean_ctor_set(x_21, 1, x_20);
x_22 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_22, 0, x_21);
lean_ctor_set_uint8(x_22, sizeof(void*)*1, x_9);
x_23 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_23, 0, x_18);
lean_ctor_set(x_23, 1, x_22);
x_24 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20);
x_25 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__21));
x_26 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_26, 0, x_25);
lean_ctor_set(x_26, 1, x_23);
x_27 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__22));
x_28 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_28, 0, x_26);
lean_ctor_set(x_28, 1, x_27);
x_29 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_29, 0, x_24);
lean_ctor_set(x_29, 1, x_28);
x_30 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_30, 0, x_29);
lean_ctor_set_uint8(x_30, sizeof(void*)*1, x_9);
return x_30;
}
else
{
lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; lean_object* x_35; lean_object* x_36; lean_object* x_37; uint8_t x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; lean_object* x_42; lean_object* x_43; lean_object* x_44; lean_object* x_45; lean_object* x_46; lean_object* x_47; lean_object* x_48; lean_object* x_49; lean_object* x_50; lean_object* x_51; lean_object* x_52; lean_object* x_53; lean_object* x_54; lean_object* x_55; lean_object* x_56; lean_object* x_57; lean_object* x_58; lean_object* x_59; 
x_31 = lean_ctor_get(x_1, 0);
x_32 = lean_ctor_get(x_1, 1);
lean_inc(x_32);
lean_inc(x_31);
lean_dec(x_1);
x_33 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__5));
x_34 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__3));
x_35 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4, &lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4_once, _init_lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__4);
x_36 = lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg(x_31);
x_37 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_37, 0, x_35);
lean_ctor_set(x_37, 1, x_36);
x_38 = 0;
x_39 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_39, 0, x_37);
lean_ctor_set_uint8(x_39, sizeof(void*)*1, x_38);
x_40 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_40, 0, x_34);
lean_ctor_set(x_40, 1, x_39);
x_41 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__9));
x_42 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_42, 0, x_40);
lean_ctor_set(x_42, 1, x_41);
x_43 = lean_box(1);
x_44 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_44, 0, x_42);
lean_ctor_set(x_44, 1, x_43);
x_45 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__6));
x_46 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_46, 0, x_44);
lean_ctor_set(x_46, 1, x_45);
x_47 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_47, 0, x_46);
lean_ctor_set(x_47, 1, x_33);
x_48 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7, &lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7_once, _init_lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg___closed__7);
x_49 = lp_LDIRProofs_LDIR_instReprPayloadTable_repr___redArg(x_32);
x_50 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_50, 0, x_48);
lean_ctor_set(x_50, 1, x_49);
x_51 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_51, 0, x_50);
lean_ctor_set_uint8(x_51, sizeof(void*)*1, x_38);
x_52 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_52, 0, x_47);
lean_ctor_set(x_52, 1, x_51);
x_53 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20, &lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20_once, _init_lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__20);
x_54 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__21));
x_55 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_55, 0, x_54);
lean_ctor_set(x_55, 1, x_52);
x_56 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprSIRInstruction_repr___redArg___closed__22));
x_57 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_57, 0, x_55);
lean_ctor_set(x_57, 1, x_56);
x_58 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_58, 0, x_53);
lean_ctor_set(x_58, 1, x_57);
x_59 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_59, 0, x_58);
lean_ctor_set_uint8(x_59, sizeof(void*)*1, x_38);
return x_59;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___redArg(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprSIRDocumentWithPayload_repr(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___redArg(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_List_repr___at___00LDIR_instReprSIRDocumentWithPayload_repr_spec__0(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_List_beq___at___00LDIR_instBEqSIRDocumentWithPayload_beq_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
if (lean_obj_tag(x_2) == 0)
{
uint8_t x_3; 
x_3 = 1;
return x_3;
}
else
{
uint8_t x_4; 
x_4 = 0;
return x_4;
}
}
else
{
if (lean_obj_tag(x_2) == 0)
{
uint8_t x_5; 
x_5 = 0;
return x_5;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; uint8_t x_10; 
x_6 = lean_ctor_get(x_1, 0);
x_7 = lean_ctor_get(x_1, 1);
x_8 = lean_ctor_get(x_2, 0);
x_9 = lean_ctor_get(x_2, 1);
x_10 = lp_LDIRProofs_LDIR_instBEqSIRInstruction_beq(x_6, x_8);
if (x_10 == 0)
{
return x_10;
}
else
{
x_1 = x_7;
x_2 = x_9;
goto _start;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_beq___at___00LDIR_instBEqSIRDocumentWithPayload_beq_spec__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_List_beq___at___00LDIR_instBEqSIRDocumentWithPayload_beq_spec__0(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload_beq(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; uint8_t x_7; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_1, 1);
x_5 = lean_ctor_get(x_2, 0);
x_6 = lean_ctor_get(x_2, 1);
x_7 = lp_LDIRProofs_List_beq___at___00LDIR_instBEqSIRDocumentWithPayload_beq_spec__0(x_3, x_5);
if (x_7 == 0)
{
return x_7;
}
else
{
uint8_t x_8; 
x_8 = lean_string_dec_eq(x_4, x_6);
return x_8;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instBEqSIRDocumentWithPayload_beq(x_1, x_2);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_GIR__COMMAND__ARGS(void) {
_start:
{
lean_object* x_1; 
x_1 = lean_unsigned_to_nat(8u);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(uint8_t x_1) {
_start:
{
switch (x_1) {
case 0:
{
lean_object* x_2; 
x_2 = lean_unsigned_to_nat(0u);
return x_2;
}
case 1:
{
lean_object* x_3; 
x_3 = lean_unsigned_to_nat(1u);
return x_3;
}
case 2:
{
lean_object* x_4; 
x_4 = lean_unsigned_to_nat(2u);
return x_4;
}
case 3:
{
lean_object* x_5; 
x_5 = lean_unsigned_to_nat(3u);
return x_5;
}
case 4:
{
lean_object* x_6; 
x_6 = lean_unsigned_to_nat(4u);
return x_6;
}
case 5:
{
lean_object* x_7; 
x_7 = lean_unsigned_to_nat(5u);
return x_7;
}
default: 
{
lean_object* x_8; 
x_8 = lean_unsigned_to_nat(6u);
return x_8;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorIdx___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lean_unbox(x_1);
x_3 = lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_toCtorIdx(uint8_t x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_toCtorIdx___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lean_unbox(x_1);
x_3 = lp_LDIRProofs_LDIR_GIROpcode_toCtorIdx(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_ctorElim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim(lean_object* x_1, lean_object* x_2, uint8_t x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
lean_inc(x_5);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ctorElim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
uint8_t x_6; lean_object* x_7; 
x_6 = lean_unbox(x_3);
x_7 = lp_LDIRProofs_LDIR_GIROpcode_ctorElim(x_1, x_2, x_6, x_4, x_5);
lean_dec(x_5);
lean_dec(x_2);
return x_7;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_setFont_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_setFont_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_GIROpcode_setFont_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_GIROpcode_moveXY_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_GIROpcode_putGlyph_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_GIROpcode_drawRule_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_GIROpcode_pushStack_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_popStack_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_popStack_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_GIROpcode_popStack_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_inc(x_4);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_2);
x_6 = lp_LDIRProofs_LDIR_GIROpcode_attachMetadata_elim(x_1, x_5, x_3, x_4);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr(uint8_t x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_10; lean_object* x_17; lean_object* x_24; lean_object* x_31; lean_object* x_38; lean_object* x_45; 
switch (x_1) {
case 0:
{
lean_object* x_52; uint8_t x_53; 
x_52 = lean_unsigned_to_nat(1024u);
x_53 = lean_nat_dec_le(x_52, x_2);
if (x_53 == 0)
{
lean_object* x_54; 
x_54 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_3 = x_54;
goto block_9;
}
else
{
lean_object* x_55; 
x_55 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_3 = x_55;
goto block_9;
}
}
case 1:
{
lean_object* x_56; uint8_t x_57; 
x_56 = lean_unsigned_to_nat(1024u);
x_57 = lean_nat_dec_le(x_56, x_2);
if (x_57 == 0)
{
lean_object* x_58; 
x_58 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_10 = x_58;
goto block_16;
}
else
{
lean_object* x_59; 
x_59 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_10 = x_59;
goto block_16;
}
}
case 2:
{
lean_object* x_60; uint8_t x_61; 
x_60 = lean_unsigned_to_nat(1024u);
x_61 = lean_nat_dec_le(x_60, x_2);
if (x_61 == 0)
{
lean_object* x_62; 
x_62 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_17 = x_62;
goto block_23;
}
else
{
lean_object* x_63; 
x_63 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_17 = x_63;
goto block_23;
}
}
case 3:
{
lean_object* x_64; uint8_t x_65; 
x_64 = lean_unsigned_to_nat(1024u);
x_65 = lean_nat_dec_le(x_64, x_2);
if (x_65 == 0)
{
lean_object* x_66; 
x_66 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_24 = x_66;
goto block_30;
}
else
{
lean_object* x_67; 
x_67 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_24 = x_67;
goto block_30;
}
}
case 4:
{
lean_object* x_68; uint8_t x_69; 
x_68 = lean_unsigned_to_nat(1024u);
x_69 = lean_nat_dec_le(x_68, x_2);
if (x_69 == 0)
{
lean_object* x_70; 
x_70 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_31 = x_70;
goto block_37;
}
else
{
lean_object* x_71; 
x_71 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_31 = x_71;
goto block_37;
}
}
case 5:
{
lean_object* x_72; uint8_t x_73; 
x_72 = lean_unsigned_to_nat(1024u);
x_73 = lean_nat_dec_le(x_72, x_2);
if (x_73 == 0)
{
lean_object* x_74; 
x_74 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_38 = x_74;
goto block_44;
}
else
{
lean_object* x_75; 
x_75 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_38 = x_75;
goto block_44;
}
}
default: 
{
lean_object* x_76; uint8_t x_77; 
x_76 = lean_unsigned_to_nat(1024u);
x_77 = lean_nat_dec_le(x_76, x_2);
if (x_77 == 0)
{
lean_object* x_78; 
x_78 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__30);
x_45 = x_78;
goto block_51;
}
else
{
lean_object* x_79; 
x_79 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31, &lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31_once, _init_lp_LDIRProofs_LDIR_instReprBlockType_repr___closed__31);
x_45 = x_79;
goto block_51;
}
}
}
block_9:
{
lean_object* x_4; lean_object* x_5; uint8_t x_6; lean_object* x_7; lean_object* x_8; 
x_4 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__1));
x_5 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_5, 0, x_3);
lean_ctor_set(x_5, 1, x_4);
x_6 = 0;
x_7 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_7, 0, x_5);
lean_ctor_set_uint8(x_7, sizeof(void*)*1, x_6);
x_8 = l_Repr_addAppParen(x_7, x_2);
return x_8;
}
block_16:
{
lean_object* x_11; lean_object* x_12; uint8_t x_13; lean_object* x_14; lean_object* x_15; 
x_11 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__3));
x_12 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_12, 0, x_10);
lean_ctor_set(x_12, 1, x_11);
x_13 = 0;
x_14 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_14, 0, x_12);
lean_ctor_set_uint8(x_14, sizeof(void*)*1, x_13);
x_15 = l_Repr_addAppParen(x_14, x_2);
return x_15;
}
block_23:
{
lean_object* x_18; lean_object* x_19; uint8_t x_20; lean_object* x_21; lean_object* x_22; 
x_18 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__5));
x_19 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_19, 0, x_17);
lean_ctor_set(x_19, 1, x_18);
x_20 = 0;
x_21 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_21, 0, x_19);
lean_ctor_set_uint8(x_21, sizeof(void*)*1, x_20);
x_22 = l_Repr_addAppParen(x_21, x_2);
return x_22;
}
block_30:
{
lean_object* x_25; lean_object* x_26; uint8_t x_27; lean_object* x_28; lean_object* x_29; 
x_25 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__7));
x_26 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_26, 0, x_24);
lean_ctor_set(x_26, 1, x_25);
x_27 = 0;
x_28 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_28, 0, x_26);
lean_ctor_set_uint8(x_28, sizeof(void*)*1, x_27);
x_29 = l_Repr_addAppParen(x_28, x_2);
return x_29;
}
block_37:
{
lean_object* x_32; lean_object* x_33; uint8_t x_34; lean_object* x_35; lean_object* x_36; 
x_32 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__9));
x_33 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_33, 0, x_31);
lean_ctor_set(x_33, 1, x_32);
x_34 = 0;
x_35 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_35, 0, x_33);
lean_ctor_set_uint8(x_35, sizeof(void*)*1, x_34);
x_36 = l_Repr_addAppParen(x_35, x_2);
return x_36;
}
block_44:
{
lean_object* x_39; lean_object* x_40; uint8_t x_41; lean_object* x_42; lean_object* x_43; 
x_39 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__11));
x_40 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_40, 0, x_38);
lean_ctor_set(x_40, 1, x_39);
x_41 = 0;
x_42 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_42, 0, x_40);
lean_ctor_set_uint8(x_42, sizeof(void*)*1, x_41);
x_43 = l_Repr_addAppParen(x_42, x_2);
return x_43;
}
block_51:
{
lean_object* x_46; lean_object* x_47; uint8_t x_48; lean_object* x_49; lean_object* x_50; 
x_46 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprGIROpcode_repr___closed__13));
x_47 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_47, 0, x_45);
lean_ctor_set(x_47, 1, x_46);
x_48 = 0;
x_49 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_49, 0, x_47);
lean_ctor_set_uint8(x_49, sizeof(void*)*1, x_48);
x_50 = l_Repr_addAppParen(x_49, x_2);
return x_50;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprGIROpcode_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lean_unbox(x_1);
x_4 = lp_LDIRProofs_LDIR_instReprGIROpcode_repr(x_3, x_2);
lean_dec(x_2);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqGIROpcode_beq(uint8_t x_1, uint8_t x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(x_1);
x_4 = lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(x_2);
x_5 = lean_nat_dec_eq(x_3, x_4);
lean_dec(x_4);
lean_dec(x_3);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqGIROpcode_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; uint8_t x_4; uint8_t x_5; lean_object* x_6; 
x_3 = lean_unbox(x_1);
x_4 = lean_unbox(x_2);
x_5 = lp_LDIRProofs_LDIR_instBEqGIROpcode_beq(x_3, x_4);
x_6 = lean_box(x_5);
return x_6;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_GIROpcode_ofNat(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
x_2 = lean_unsigned_to_nat(2u);
x_3 = lean_nat_dec_le(x_1, x_2);
if (x_3 == 0)
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(4u);
x_5 = lean_nat_dec_le(x_1, x_4);
if (x_5 == 0)
{
lean_object* x_6; uint8_t x_7; 
x_6 = lean_unsigned_to_nat(5u);
x_7 = lean_nat_dec_le(x_1, x_6);
if (x_7 == 0)
{
uint8_t x_8; 
x_8 = 6;
return x_8;
}
else
{
uint8_t x_9; 
x_9 = 5;
return x_9;
}
}
else
{
lean_object* x_10; uint8_t x_11; 
x_10 = lean_unsigned_to_nat(3u);
x_11 = lean_nat_dec_le(x_1, x_10);
if (x_11 == 0)
{
uint8_t x_12; 
x_12 = 4;
return x_12;
}
else
{
uint8_t x_13; 
x_13 = 3;
return x_13;
}
}
}
else
{
lean_object* x_14; uint8_t x_15; 
x_14 = lean_unsigned_to_nat(0u);
x_15 = lean_nat_dec_le(x_1, x_14);
if (x_15 == 0)
{
lean_object* x_16; uint8_t x_17; 
x_16 = lean_unsigned_to_nat(1u);
x_17 = lean_nat_dec_le(x_1, x_16);
if (x_17 == 0)
{
uint8_t x_18; 
x_18 = 2;
return x_18;
}
else
{
uint8_t x_19; 
x_19 = 1;
return x_19;
}
}
else
{
uint8_t x_20; 
x_20 = 0;
return x_20;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIROpcode_ofNat___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_GIROpcode_ofNat(x_1);
lean_dec(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instDecidableEqGIROpcode(uint8_t x_1, uint8_t x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(x_1);
x_4 = lp_LDIRProofs_LDIR_GIROpcode_ctorIdx(x_2);
x_5 = lean_nat_dec_eq(x_3, x_4);
lean_dec(x_4);
lean_dec(x_3);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instDecidableEqGIROpcode___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; uint8_t x_4; uint8_t x_5; lean_object* x_6; 
x_3 = lean_unbox(x_1);
x_4 = lean_unbox(x_2);
x_5 = lp_LDIRProofs_LDIR_instDecidableEqGIROpcode(x_3, x_4);
x_6 = lean_box(x_5);
return x_6;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___closed__0(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(0u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lean_obj_once(&lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___closed__0, &lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___closed__0_once, _init_lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___closed__0);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_GIRCommand_zeroed___lam__0(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed(uint8_t x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; 
x_2 = ((lean_object*)(lp_LDIRProofs_LDIR_GIRCommand_zeroed___closed__0));
x_3 = lean_alloc_ctor(0, 1, 1);
lean_ctor_set(x_3, 0, x_2);
lean_ctor_set_uint8(x_3, sizeof(void*)*1, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_GIRCommand_zeroed___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lean_unbox(x_1);
x_3 = lp_LDIRProofs_LDIR_GIRCommand_zeroed(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__0(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; lean_object* x_5; uint8_t x_6; 
lean_inc(x_3);
x_4 = lean_apply_1(x_1, x_3);
x_5 = lean_apply_1(x_2, x_3);
x_6 = lean_int_dec_eq(x_4, x_5);
lean_dec(x_5);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__0___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
uint8_t x_4; lean_object* x_5; 
x_4 = lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__0(x_1, x_2, x_3);
x_5 = lean_box(x_4);
return x_5;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__1(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; uint8_t x_5; lean_object* x_6; uint8_t x_7; 
x_3 = lean_ctor_get_uint8(x_1, sizeof(void*)*1);
x_4 = lean_ctor_get(x_1, 0);
lean_inc_ref(x_4);
lean_dec_ref(x_1);
x_5 = lean_ctor_get_uint8(x_2, sizeof(void*)*1);
x_6 = lean_ctor_get(x_2, 0);
lean_inc_ref(x_6);
lean_dec_ref(x_2);
x_7 = lp_LDIRProofs_LDIR_instBEqGIROpcode_beq(x_3, x_5);
if (x_7 == 0)
{
lean_dec_ref(x_6);
lean_dec_ref(x_4);
return x_7;
}
else
{
lean_object* x_8; lean_object* x_9; uint8_t x_10; 
x_8 = lean_alloc_closure((void*)(lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__0___boxed), 3, 2);
lean_closure_set(x_8, 0, x_4);
lean_closure_set(x_8, 1, x_6);
x_9 = lean_unsigned_to_nat(8u);
x_10 = l_Nat_decidableForallFin___redArg(x_9, x_8);
return x_10;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__1___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instBEqGIRCommand___lam__1(x_1, x_2);
x_4 = lean_box(x_3);
return x_4;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0(void) {
_start:
{
uint8_t x_1; lean_object* x_2; 
x_1 = 0;
x_2 = lp_LDIRProofs_LDIR_GIRCommand_zeroed(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instInhabitedGIRCommand(void) {
_start:
{
lean_object* x_1; 
x_1 = lean_obj_once(&lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0, &lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0_once, _init_lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_entityUnique_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_3; 
x_3 = l_List_reverse___redArg(x_2);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_1);
if (x_4 == 0)
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; 
x_5 = lean_ctor_get(x_1, 0);
x_6 = lean_ctor_get(x_1, 1);
x_7 = lean_ctor_get(x_5, 1);
lean_inc(x_7);
lean_dec(x_5);
lean_ctor_set(x_1, 1, x_2);
lean_ctor_set(x_1, 0, x_7);
{
lean_object* _tmp_0 = x_6;
lean_object* _tmp_1 = x_1;
x_1 = _tmp_0;
x_2 = _tmp_1;
}
goto _start;
}
else
{
lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; 
x_9 = lean_ctor_get(x_1, 0);
x_10 = lean_ctor_get(x_1, 1);
lean_inc(x_10);
lean_inc(x_9);
lean_dec(x_1);
x_11 = lean_ctor_get(x_9, 1);
lean_inc(x_11);
lean_dec(x_9);
x_12 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_12, 0, x_11);
lean_ctor_set(x_12, 1, x_2);
x_1 = x_10;
x_2 = x_12;
goto _start;
}
}
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_entityUnique(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_2 = lean_alloc_closure((void*)(l_instDecidableEqNat___boxed), 2, 0);
x_3 = lean_box(0);
x_4 = lp_LDIRProofs_List_mapTR_loop___at___00LDIR_entityUnique_spec__0(x_1, x_3);
x_5 = l_List_nodupDecidable___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_entityUnique___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_entityUnique(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_parentIdValid___closed__0(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_alloc_closure((void*)(l_instDecidableEqNat___boxed), 2, 0);
x_2 = lean_alloc_closure((void*)(l_instBEqOfDecidableEq___redArg___lam__0___boxed), 3, 1);
lean_closure_set(x_2, 0, x_1);
return x_2;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_parentIdValid(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lean_ctor_get(x_1, 2);
lean_inc(x_3);
lean_dec_ref(x_1);
x_4 = lean_unsigned_to_nat(4294967295u);
x_5 = lean_nat_dec_eq(x_3, x_4);
if (x_5 == 0)
{
lean_object* x_6; uint8_t x_7; 
x_6 = lean_obj_once(&lp_LDIRProofs_LDIR_parentIdValid___closed__0, &lp_LDIRProofs_LDIR_parentIdValid___closed__0_once, _init_lp_LDIRProofs_LDIR_parentIdValid___closed__0);
x_7 = l_List_elem___redArg(x_6, x_3, x_2);
return x_7;
}
else
{
lean_dec(x_3);
lean_dec(x_2);
return x_5;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_parentIdValid___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_parentIdValid(x_1, x_2);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_parentExists___lam__0(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; 
x_3 = lp_LDIRProofs_LDIR_parentIdValid(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_parentExists___lam__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_parentExists___lam__0(x_1, x_2);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_parentExists(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_2 = lean_box(0);
lean_inc(x_1);
x_3 = lp_LDIRProofs_List_mapTR_loop___at___00LDIR_entityUnique_spec__0(x_1, x_2);
x_4 = lean_alloc_closure((void*)(lp_LDIRProofs_LDIR_parentExists___lam__0___boxed), 2, 1);
lean_closure_set(x_4, 0, x_3);
x_5 = l_List_all___redArg(x_1, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_parentExists___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_parentExists(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_filterTR_loop___at___00LDIR_hasSingleRoot_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_3; 
x_3 = l_List_reverse___redArg(x_2);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_1);
if (x_4 == 0)
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; uint8_t x_9; 
x_5 = lean_ctor_get(x_1, 0);
x_6 = lean_ctor_get(x_1, 1);
x_7 = lean_ctor_get(x_5, 2);
x_8 = lean_unsigned_to_nat(4294967295u);
x_9 = lean_nat_dec_eq(x_7, x_8);
if (x_9 == 0)
{
lean_free_object(x_1);
lean_dec(x_5);
x_1 = x_6;
goto _start;
}
else
{
lean_ctor_set(x_1, 1, x_2);
{
lean_object* _tmp_0 = x_6;
lean_object* _tmp_1 = x_1;
x_1 = _tmp_0;
x_2 = _tmp_1;
}
goto _start;
}
}
else
{
lean_object* x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; uint8_t x_16; 
x_12 = lean_ctor_get(x_1, 0);
x_13 = lean_ctor_get(x_1, 1);
lean_inc(x_13);
lean_inc(x_12);
lean_dec(x_1);
x_14 = lean_ctor_get(x_12, 2);
x_15 = lean_unsigned_to_nat(4294967295u);
x_16 = lean_nat_dec_eq(x_14, x_15);
if (x_16 == 0)
{
lean_dec(x_12);
x_1 = x_13;
goto _start;
}
else
{
lean_object* x_18; 
x_18 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_18, 0, x_12);
lean_ctor_set(x_18, 1, x_2);
x_1 = x_13;
x_2 = x_18;
goto _start;
}
}
}
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_hasSingleRoot(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; lean_object* x_5; uint8_t x_6; 
x_2 = lean_box(0);
x_3 = lp_LDIRProofs_List_filterTR_loop___at___00LDIR_hasSingleRoot_spec__0(x_1, x_2);
x_4 = l_List_lengthTR___redArg(x_3);
lean_dec(x_3);
x_5 = lean_unsigned_to_nat(1u);
x_6 = lean_nat_dec_eq(x_4, x_5);
lean_dec(x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_hasSingleRoot___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_hasSingleRoot(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclicAux___lam__0(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; uint8_t x_4; 
x_3 = lean_ctor_get(x_2, 1);
x_4 = lean_nat_dec_eq(x_3, x_1);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclicAux___lam__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_isAcyclicAux___lam__0(x_1, x_2);
lean_dec_ref(x_2);
lean_dec(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclicAux(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_3, x_4);
if (x_5 == 1)
{
uint8_t x_6; 
lean_dec(x_3);
lean_dec(x_2);
lean_dec(x_1);
x_6 = 0;
return x_6;
}
else
{
lean_object* x_7; lean_object* x_8; 
x_7 = lean_alloc_closure((void*)(lp_LDIRProofs_LDIR_isAcyclicAux___lam__0___boxed), 2, 1);
lean_closure_set(x_7, 0, x_2);
lean_inc(x_1);
x_8 = l_List_find_x3f___redArg(x_7, x_1);
if (lean_obj_tag(x_8) == 0)
{
uint8_t x_9; 
lean_dec(x_3);
lean_dec(x_1);
x_9 = 1;
return x_9;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; uint8_t x_13; 
x_10 = lean_ctor_get(x_8, 0);
lean_inc(x_10);
lean_dec_ref(x_8);
x_11 = lean_ctor_get(x_10, 2);
lean_inc(x_11);
lean_dec(x_10);
x_12 = lean_unsigned_to_nat(4294967295u);
x_13 = lean_nat_dec_eq(x_11, x_12);
if (x_13 == 0)
{
lean_object* x_14; lean_object* x_15; 
x_14 = lean_unsigned_to_nat(1u);
x_15 = lean_nat_sub(x_3, x_14);
lean_dec(x_3);
x_2 = x_11;
x_3 = x_15;
goto _start;
}
else
{
lean_dec(x_11);
lean_dec(x_3);
lean_dec(x_1);
return x_13;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclicAux___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
uint8_t x_4; lean_object* x_5; 
x_4 = lp_LDIRProofs_LDIR_isAcyclicAux(x_1, x_2, x_3);
x_5 = lean_box(x_4);
return x_5;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclic___lam__0(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lean_ctor_get(x_2, 1);
lean_inc(x_3);
lean_dec_ref(x_2);
x_4 = l_List_lengthTR___redArg(x_1);
x_5 = lp_LDIRProofs_LDIR_isAcyclicAux(x_1, x_3, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclic___lam__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_isAcyclic___lam__0(x_1, x_2);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_isAcyclic(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
lean_inc(x_1);
x_2 = lean_alloc_closure((void*)(lp_LDIRProofs_LDIR_isAcyclic___lam__0___boxed), 2, 1);
lean_closure_set(x_2, 0, x_1);
x_3 = l_List_all___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_isAcyclic___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_isAcyclic(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_payloadValid___lam__0(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lean_ctor_get(x_2, 0);
if (lean_obj_tag(x_3) == 2)
{
uint8_t x_4; 
x_4 = 1;
return x_4;
}
else
{
lean_object* x_5; lean_object* x_6; uint8_t x_7; 
x_5 = lean_ctor_get(x_2, 3);
x_6 = lean_string_length(x_1);
x_7 = lean_nat_dec_lt(x_5, x_6);
return x_7;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_payloadValid___lam__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_payloadValid___lam__0(x_1, x_2);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_payloadValid(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_2 = lean_ctor_get(x_1, 0);
lean_inc(x_2);
x_3 = lean_ctor_get(x_1, 1);
lean_inc_ref(x_3);
lean_dec_ref(x_1);
x_4 = lean_alloc_closure((void*)(lp_LDIRProofs_LDIR_payloadValid___lam__0___boxed), 2, 1);
lean_closure_set(x_4, 0, x_3);
x_5 = l_List_all___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_payloadValid___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_payloadValid(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_wellFormedSIR(lean_object* x_1) {
_start:
{
uint8_t x_2; uint8_t x_6; 
lean_inc(x_1);
x_6 = lp_LDIRProofs_LDIR_entityUnique(x_1);
if (x_6 == 0)
{
x_2 = x_6;
goto block_5;
}
else
{
uint8_t x_7; 
lean_inc(x_1);
x_7 = lp_LDIRProofs_LDIR_parentExists(x_1);
x_2 = x_7;
goto block_5;
}
block_5:
{
if (x_2 == 0)
{
lean_dec(x_1);
return x_2;
}
else
{
uint8_t x_3; 
lean_inc(x_1);
x_3 = lp_LDIRProofs_LDIR_isAcyclic(x_1);
if (x_3 == 0)
{
lean_dec(x_1);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = lp_LDIRProofs_LDIR_hasSingleRoot(x_1);
return x_4;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_wellFormedSIR___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_wellFormedSIR(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_wellFormedSIRWithPayload(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
x_2 = lean_ctor_get(x_1, 0);
lean_inc(x_2);
x_3 = lp_LDIRProofs_LDIR_wellFormedSIR(x_2);
if (x_3 == 0)
{
lean_dec_ref(x_1);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = lp_LDIRProofs_LDIR_payloadValid(x_1);
return x_4;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_wellFormedSIRWithPayload___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_wellFormedSIRWithPayload(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_stackBalancedAux(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_3; uint8_t x_4; 
x_3 = lean_unsigned_to_nat(0u);
x_4 = lean_nat_dec_eq(x_2, x_3);
lean_dec(x_2);
return x_4;
}
else
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_1, 0);
x_6 = lean_ctor_get_uint8(x_5, sizeof(void*)*1);
switch (x_6) {
case 4:
{
lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_7 = lean_ctor_get(x_1, 1);
x_8 = lean_unsigned_to_nat(1u);
x_9 = lean_nat_add(x_2, x_8);
lean_dec(x_2);
x_1 = x_7;
x_2 = x_9;
goto _start;
}
case 5:
{
lean_object* x_11; lean_object* x_12; uint8_t x_13; 
x_11 = lean_ctor_get(x_1, 1);
x_12 = lean_unsigned_to_nat(0u);
x_13 = lean_nat_dec_lt(x_12, x_2);
if (x_13 == 0)
{
lean_dec(x_2);
return x_13;
}
else
{
lean_object* x_14; lean_object* x_15; 
x_14 = lean_unsigned_to_nat(1u);
x_15 = lean_nat_sub(x_2, x_14);
lean_dec(x_2);
x_1 = x_11;
x_2 = x_15;
goto _start;
}
}
default: 
{
lean_object* x_17; 
x_17 = lean_ctor_get(x_1, 1);
x_1 = x_17;
goto _start;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_stackBalancedAux___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_stackBalancedAux(x_1, x_2);
lean_dec(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_stackBalanced(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
x_2 = lean_unsigned_to_nat(0u);
x_3 = lp_LDIRProofs_LDIR_stackBalancedAux(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_stackBalanced___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_stackBalanced(x_1);
lean_dec(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_pageWellFormed(lean_object* x_1) {
_start:
{
uint8_t x_2; 
x_2 = lp_LDIRProofs_LDIR_stackBalanced(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_pageWellFormed___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_pageWellFormed(x_1);
lean_dec(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_wellFormedGIR(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
x_2 = ((lean_object*)(lp_LDIRProofs_LDIR_wellFormedGIR___closed__0));
x_3 = l_List_all___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_wellFormedGIR___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lp_LDIRProofs_LDIR_wellFormedGIR(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileStub(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = ((lean_object*)(lp_LDIRProofs_LDIR_compileStub___closed__0));
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileStub___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_compileStub(x_1);
lean_dec(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__0(void) {
_start:
{
uint8_t x_1; lean_object* x_2; 
x_1 = 2;
x_2 = lp_LDIRProofs_LDIR_GIRCommand_zeroed(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__1(void) {
_start:
{
lean_object* x_1; lean_object* x_2; lean_object* x_3; 
x_1 = lean_box(0);
x_2 = lean_obj_once(&lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__0, &lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__0_once, _init_lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__0);
x_3 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_3, 0, x_2);
lean_ctor_set(x_3, 1, x_1);
return x_3;
}
}
static lean_object* _init_lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__2(void) {
_start:
{
lean_object* x_1; lean_object* x_2; lean_object* x_3; 
x_1 = lean_box(0);
x_2 = lean_obj_once(&lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0, &lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0_once, _init_lp_LDIRProofs_LDIR_instInhabitedGIRCommand___closed__0);
x_3 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_3, 0, x_2);
lean_ctor_set(x_3, 1, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lean_ctor_get(x_2, 0);
switch (lean_obj_tag(x_3)) {
case 1:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lean_obj_once(&lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__1, &lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__1_once, _init_lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__1);
x_5 = l_List_appendTR___redArg(x_1, x_4);
return x_5;
}
case 0:
{
uint8_t x_6; 
x_6 = lean_ctor_get_uint8(x_3, 0);
if (x_6 == 2)
{
lean_object* x_7; lean_object* x_8; 
x_7 = lean_obj_once(&lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__2, &lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__2_once, _init_lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___closed__2);
x_8 = l_List_appendTR___redArg(x_1, x_7);
return x_8;
}
else
{
return x_1;
}
}
default: 
{
return x_1;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep(x_1, x_2);
lean_dec_ref(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_compileReal_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
return x_1;
}
else
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; 
x_3 = lean_ctor_get(x_2, 0);
x_4 = lean_ctor_get(x_2, 1);
x_5 = lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_compileStep(x_1, x_3);
x_1 = x_5;
x_2 = x_4;
goto _start;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_compileReal_spec__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_List_foldl___at___00LDIR_compileReal_spec__0(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileReal(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; uint8_t x_4; 
x_2 = lean_box(0);
x_3 = lp_LDIRProofs_List_foldl___at___00LDIR_compileReal_spec__0(x_2, x_1);
x_4 = l_List_isEmpty___redArg(x_3);
if (x_4 == 0)
{
lean_object* x_5; 
x_5 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_5, 0, x_3);
lean_ctor_set(x_5, 1, x_2);
return x_5;
}
else
{
lean_object* x_6; 
lean_dec(x_3);
x_6 = ((lean_object*)(lp_LDIRProofs_LDIR_compileStub___closed__0));
return x_6;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_compileReal___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_compileReal(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__4_splitter___redArg(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_4; lean_object* x_5; 
lean_dec(x_3);
x_4 = lean_box(0);
x_5 = lean_apply_1(x_2, x_4);
return x_5;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = lean_ctor_get(x_1, 0);
lean_inc(x_6);
x_7 = lean_ctor_get(x_1, 1);
lean_inc(x_7);
lean_dec_ref(x_1);
x_8 = lean_apply_2(x_3, x_6, x_7);
return x_8;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__4_splitter(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_5; lean_object* x_6; 
lean_dec(x_4);
x_5 = lean_box(0);
x_6 = lean_apply_1(x_3, x_5);
return x_6;
}
else
{
lean_object* x_7; lean_object* x_8; lean_object* x_9; 
lean_dec(x_3);
x_7 = lean_ctor_get(x_2, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_2, 1);
lean_inc(x_8);
lean_dec_ref(x_2);
x_9 = lean_apply_2(x_4, x_7, x_8);
return x_9;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter___redArg(uint8_t x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
switch (x_1) {
case 4:
{
lean_object* x_5; lean_object* x_6; 
lean_dec(x_4);
lean_dec(x_3);
x_5 = lean_box(0);
x_6 = lean_apply_1(x_2, x_5);
return x_6;
}
case 5:
{
lean_object* x_7; lean_object* x_8; 
lean_dec(x_4);
lean_dec(x_2);
x_7 = lean_box(0);
x_8 = lean_apply_1(x_3, x_7);
return x_8;
}
default: 
{
lean_object* x_9; lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_9 = lean_box(x_1);
x_10 = lean_apply_3(x_4, x_9, lean_box(0), lean_box(0));
return x_10;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter___redArg___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; lean_object* x_6; 
x_5 = lean_unbox(x_1);
x_6 = lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter___redArg(x_5, x_2, x_3, x_4);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter(lean_object* x_1, uint8_t x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
switch (x_2) {
case 4:
{
lean_object* x_6; lean_object* x_7; 
lean_dec(x_5);
lean_dec(x_4);
x_6 = lean_box(0);
x_7 = lean_apply_1(x_3, x_6);
return x_7;
}
case 5:
{
lean_object* x_8; lean_object* x_9; 
lean_dec(x_5);
lean_dec(x_3);
x_8 = lean_box(0);
x_9 = lean_apply_1(x_4, x_8);
return x_9;
}
default: 
{
lean_object* x_10; lean_object* x_11; 
lean_dec(x_4);
lean_dec(x_3);
x_10 = lean_box(x_2);
x_11 = lean_apply_3(x_5, x_10, lean_box(0), lean_box(0));
return x_11;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
uint8_t x_6; lean_object* x_7; 
x_6 = lean_unbox(x_2);
x_7 = lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_stackBalancedAux_match__1_splitter(x_1, x_6, x_3, x_4, x_5);
return x_7;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__List_any_match__1_splitter___redArg(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_5; 
lean_dec(x_4);
x_5 = lean_apply_1(x_3, x_2);
return x_5;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_3);
x_6 = lean_ctor_get(x_1, 0);
lean_inc(x_6);
x_7 = lean_ctor_get(x_1, 1);
lean_inc(x_7);
lean_dec_ref(x_1);
x_8 = lean_apply_3(x_4, x_6, x_7, x_2);
return x_8;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__List_any_match__1_splitter(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5, lean_object* x_6) {
_start:
{
if (lean_obj_tag(x_3) == 0)
{
lean_object* x_7; 
lean_dec(x_6);
x_7 = lean_apply_1(x_5, x_4);
return x_7;
}
else
{
lean_object* x_8; lean_object* x_9; lean_object* x_10; 
lean_dec(x_5);
x_8 = lean_ctor_get(x_3, 0);
lean_inc(x_8);
x_9 = lean_ctor_get(x_3, 1);
lean_inc(x_9);
lean_dec_ref(x_3);
x_10 = lean_apply_3(x_6, x_8, x_9, x_4);
return x_10;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter___redArg(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_1, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; 
lean_dec(x_3);
x_6 = lean_box(0);
x_7 = lean_apply_1(x_2, x_6);
return x_7;
}
else
{
lean_object* x_8; lean_object* x_9; lean_object* x_10; 
lean_dec(x_2);
x_8 = lean_unsigned_to_nat(1u);
x_9 = lean_nat_sub(x_1, x_8);
x_10 = lean_apply_1(x_3, x_9);
return x_10;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter___redArg___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
x_4 = lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter___redArg(x_1, x_2, x_3);
lean_dec(x_1);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_unsigned_to_nat(0u);
x_6 = lean_nat_dec_eq(x_2, x_5);
if (x_6 == 1)
{
lean_object* x_7; lean_object* x_8; 
lean_dec(x_4);
x_7 = lean_box(0);
x_8 = lean_apply_1(x_3, x_7);
return x_8;
}
else
{
lean_object* x_9; lean_object* x_10; lean_object* x_11; 
lean_dec(x_3);
x_9 = lean_unsigned_to_nat(1u);
x_10 = lean_nat_sub(x_2, x_9);
x_11 = lean_apply_1(x_4, x_10);
return x_11;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__3_splitter(x_1, x_2, x_3, x_4);
lean_dec(x_2);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__1_splitter___redArg(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_4; lean_object* x_5; 
lean_dec(x_3);
x_4 = lean_box(0);
x_5 = lean_apply_1(x_2, x_4);
return x_5;
}
else
{
lean_object* x_6; lean_object* x_7; 
lean_dec(x_2);
x_6 = lean_ctor_get(x_1, 0);
lean_inc(x_6);
lean_dec_ref(x_1);
x_7 = lean_apply_1(x_3, x_6);
return x_7;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_isAcyclicAux_match__1_splitter(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_5; lean_object* x_6; 
lean_dec(x_4);
x_5 = lean_box(0);
x_6 = lean_apply_1(x_3, x_5);
return x_6;
}
else
{
lean_object* x_7; lean_object* x_8; 
lean_dec(x_3);
x_7 = lean_ctor_get(x_2, 0);
lean_inc(x_7);
lean_dec_ref(x_2);
x_8 = lean_apply_1(x_4, x_7);
return x_8;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_payloadValid_match__1_splitter___redArg(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_1) == 2)
{
lean_object* x_4; lean_object* x_5; 
lean_dec(x_3);
x_4 = lean_box(0);
x_5 = lean_apply_1(x_2, x_4);
return x_5;
}
else
{
lean_object* x_6; 
lean_dec(x_2);
x_6 = lean_apply_2(x_3, x_1, lean_box(0));
return x_6;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_proof__ir__wellformedness_0__LDIR_payloadValid_match__1_splitter(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
if (lean_obj_tag(x_2) == 2)
{
lean_object* x_5; lean_object* x_6; 
lean_dec(x_4);
x_5 = lean_box(0);
x_6 = lean_apply_1(x_3, x_5);
return x_6;
}
else
{
lean_object* x_7; 
lean_dec(x_3);
x_7 = lean_apply_2(x_4, x_2, lean_box(0));
return x_7;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_sirSemanticContent_spec__0(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_4; 
x_4 = l_List_reverse___redArg(x_3);
return x_4;
}
else
{
uint8_t x_5; 
x_5 = !lean_is_exclusive(x_2);
if (x_5 == 0)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; lean_object* x_23; 
x_6 = lean_ctor_get(x_2, 0);
x_7 = lean_ctor_get(x_2, 1);
x_8 = lean_ctor_get(x_6, 0);
lean_inc(x_8);
x_9 = lean_ctor_get(x_6, 2);
lean_inc(x_9);
x_10 = lean_ctor_get(x_6, 3);
lean_inc(x_10);
lean_dec(x_6);
x_11 = lean_ctor_get(x_1, 1);
x_12 = lean_unsigned_to_nat(0u);
x_13 = lean_string_utf8_byte_size(x_11);
lean_inc_ref(x_11);
x_14 = lean_alloc_ctor(0, 3, 0);
lean_ctor_set(x_14, 0, x_11);
lean_ctor_set(x_14, 1, x_12);
lean_ctor_set(x_14, 2, x_13);
x_15 = l_String_Slice_Pos_nextn(x_14, x_12, x_10);
lean_dec_ref(x_14);
lean_inc(x_15);
lean_inc_ref(x_11);
x_16 = lean_alloc_ctor(0, 3, 0);
lean_ctor_set(x_16, 0, x_11);
lean_ctor_set(x_16, 1, x_15);
lean_ctor_set(x_16, 2, x_13);
x_17 = lean_unsigned_to_nat(256u);
x_18 = l_String_Slice_Pos_nextn(x_16, x_12, x_17);
lean_dec_ref(x_16);
x_19 = lean_nat_add(x_15, x_18);
lean_dec(x_18);
lean_inc_ref(x_11);
x_20 = lean_alloc_ctor(0, 3, 0);
lean_ctor_set(x_20, 0, x_11);
lean_ctor_set(x_20, 1, x_15);
lean_ctor_set(x_20, 2, x_19);
x_21 = l_String_Slice_toString(x_20);
lean_dec_ref(x_20);
x_22 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_22, 0, x_21);
lean_ctor_set(x_22, 1, x_9);
x_23 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_23, 0, x_8);
lean_ctor_set(x_23, 1, x_22);
lean_ctor_set(x_2, 1, x_3);
lean_ctor_set(x_2, 0, x_23);
{
lean_object* _tmp_1 = x_7;
lean_object* _tmp_2 = x_2;
x_2 = _tmp_1;
x_3 = _tmp_2;
}
goto _start;
}
else
{
lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; lean_object* x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; lean_object* x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; lean_object* x_42; lean_object* x_43; 
x_25 = lean_ctor_get(x_2, 0);
x_26 = lean_ctor_get(x_2, 1);
lean_inc(x_26);
lean_inc(x_25);
lean_dec(x_2);
x_27 = lean_ctor_get(x_25, 0);
lean_inc(x_27);
x_28 = lean_ctor_get(x_25, 2);
lean_inc(x_28);
x_29 = lean_ctor_get(x_25, 3);
lean_inc(x_29);
lean_dec(x_25);
x_30 = lean_ctor_get(x_1, 1);
x_31 = lean_unsigned_to_nat(0u);
x_32 = lean_string_utf8_byte_size(x_30);
lean_inc_ref(x_30);
x_33 = lean_alloc_ctor(0, 3, 0);
lean_ctor_set(x_33, 0, x_30);
lean_ctor_set(x_33, 1, x_31);
lean_ctor_set(x_33, 2, x_32);
x_34 = l_String_Slice_Pos_nextn(x_33, x_31, x_29);
lean_dec_ref(x_33);
lean_inc(x_34);
lean_inc_ref(x_30);
x_35 = lean_alloc_ctor(0, 3, 0);
lean_ctor_set(x_35, 0, x_30);
lean_ctor_set(x_35, 1, x_34);
lean_ctor_set(x_35, 2, x_32);
x_36 = lean_unsigned_to_nat(256u);
x_37 = l_String_Slice_Pos_nextn(x_35, x_31, x_36);
lean_dec_ref(x_35);
x_38 = lean_nat_add(x_34, x_37);
lean_dec(x_37);
lean_inc_ref(x_30);
x_39 = lean_alloc_ctor(0, 3, 0);
lean_ctor_set(x_39, 0, x_30);
lean_ctor_set(x_39, 1, x_34);
lean_ctor_set(x_39, 2, x_38);
x_40 = l_String_Slice_toString(x_39);
lean_dec_ref(x_39);
x_41 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_41, 0, x_40);
lean_ctor_set(x_41, 1, x_28);
x_42 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_42, 0, x_27);
lean_ctor_set(x_42, 1, x_41);
x_43 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_43, 0, x_42);
lean_ctor_set(x_43, 1, x_3);
x_2 = x_26;
x_3 = x_43;
goto _start;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_sirSemanticContent_spec__0___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
x_4 = lp_LDIRProofs_List_mapTR_loop___at___00LDIR_sirSemanticContent_spec__0(x_1, x_2, x_3);
lean_dec_ref(x_1);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_sirSemanticContent(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; 
x_2 = lean_ctor_get(x_1, 0);
lean_inc(x_2);
x_3 = lean_box(0);
x_4 = lp_LDIRProofs_List_mapTR_loop___at___00LDIR_sirSemanticContent_spec__0(x_1, x_2, x_3);
lean_dec_ref(x_1);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__0(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_4; 
lean_dec_ref(x_1);
x_4 = l_List_reverse___redArg(x_3);
return x_4;
}
else
{
uint8_t x_5; 
x_5 = !lean_is_exclusive(x_2);
if (x_5 == 0)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_2, 0);
x_7 = lean_ctor_get(x_2, 1);
x_8 = lean_ctor_get(x_1, 0);
lean_inc_ref(x_8);
x_9 = lean_apply_1(x_8, x_6);
lean_ctor_set(x_2, 1, x_3);
lean_ctor_set(x_2, 0, x_9);
{
lean_object* _tmp_1 = x_7;
lean_object* _tmp_2 = x_2;
x_2 = _tmp_1;
x_3 = _tmp_2;
}
goto _start;
}
else
{
lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_11 = lean_ctor_get(x_2, 0);
x_12 = lean_ctor_get(x_2, 1);
lean_inc(x_12);
lean_inc(x_11);
lean_dec(x_2);
x_13 = lean_ctor_get(x_1, 0);
lean_inc_ref(x_13);
x_14 = lean_apply_1(x_13, x_11);
x_15 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_15, 0, x_14);
lean_ctor_set(x_15, 1, x_3);
x_2 = x_12;
x_3 = x_15;
goto _start;
}
}
}
}
static lean_object* _init_lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(8u);
x_2 = l_List_finRange(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_3; 
x_3 = l_List_reverse___redArg(x_2);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_1);
if (x_4 == 0)
{
lean_object* x_5; lean_object* x_6; uint8_t x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; 
x_5 = lean_ctor_get(x_1, 0);
x_6 = lean_ctor_get(x_1, 1);
x_7 = lean_ctor_get_uint8(x_5, sizeof(void*)*1);
x_8 = lean_obj_once(&lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0, &lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0_once, _init_lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0);
x_9 = lean_box(0);
x_10 = lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__0(x_5, x_8, x_9);
x_11 = lean_box(x_7);
x_12 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_12, 0, x_11);
lean_ctor_set(x_12, 1, x_10);
lean_ctor_set(x_1, 1, x_2);
lean_ctor_set(x_1, 0, x_12);
{
lean_object* _tmp_0 = x_6;
lean_object* _tmp_1 = x_1;
x_1 = _tmp_0;
x_2 = _tmp_1;
}
goto _start;
}
else
{
lean_object* x_14; lean_object* x_15; uint8_t x_16; lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_14 = lean_ctor_get(x_1, 0);
x_15 = lean_ctor_get(x_1, 1);
lean_inc(x_15);
lean_inc(x_14);
lean_dec(x_1);
x_16 = lean_ctor_get_uint8(x_14, sizeof(void*)*1);
x_17 = lean_obj_once(&lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0, &lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0_once, _init_lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1___closed__0);
x_18 = lean_box(0);
x_19 = lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__0(x_14, x_17, x_18);
x_20 = lean_box(x_16);
x_21 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_19);
x_22 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_22, 0, x_21);
lean_ctor_set(x_22, 1, x_2);
x_1 = x_15;
x_2 = x_22;
goto _start;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_girSemanticContent_spec__2(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
return x_1;
}
else
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; 
x_3 = lean_ctor_get(x_2, 0);
lean_inc(x_3);
x_4 = lean_ctor_get(x_2, 1);
lean_inc(x_4);
lean_dec_ref(x_2);
x_5 = lean_box(0);
x_6 = lp_LDIRProofs_List_mapTR_loop___at___00LDIR_girSemanticContent_spec__1(x_3, x_5);
x_7 = l_List_appendTR___redArg(x_1, x_6);
x_1 = x_7;
x_2 = x_4;
goto _start;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_girSemanticContent(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; 
x_2 = lean_box(0);
x_3 = lp_LDIRProofs_List_foldl___at___00LDIR_girSemanticContent_spec__2(x_2, x_1);
return x_3;
}
}
lean_object* initialize_Init(uint8_t builtin);
lean_object* initialize_Init(uint8_t builtin);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_LDIRProofs_LDIRProofs_proof__ir__wellformedness(uint8_t builtin) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_Init(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
lp_LDIRProofs_LDIR_rootSentinel = _init_lp_LDIRProofs_LDIR_rootSentinel();
lean_mark_persistent(lp_LDIRProofs_LDIR_rootSentinel);
lp_LDIRProofs_LDIR_GIR__COMMAND__ARGS = _init_lp_LDIRProofs_LDIR_GIR__COMMAND__ARGS();
lean_mark_persistent(lp_LDIRProofs_LDIR_GIR__COMMAND__ARGS);
lp_LDIRProofs_LDIR_instInhabitedGIRCommand = _init_lp_LDIRProofs_LDIR_instInhabitedGIRCommand();
lean_mark_persistent(lp_LDIRProofs_LDIR_instInhabitedGIRCommand);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
