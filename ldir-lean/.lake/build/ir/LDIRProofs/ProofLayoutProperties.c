// Lean compiler output
// Module: LDIRProofs.ProofLayoutProperties
// Imports: public import Init public import Mathlib.Data.List.Basic public import Mathlib.Data.List.Lemmas public import Mathlib.Tactic public import LDIRProofs.proof_ir_wellformedness
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
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorIdx(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorIdx___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorElim(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorElim___boxed(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_box_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_box_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_glue_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_glue_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_penalty_elim___redArg(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_penalty_elim(lean_object*, lean_object*, lean_object*, lean_object*);
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 16, .m_capacity = 16, .m_length = 15, .m_data = "LDIR.KPItem.box"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__0_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__1_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__1_value),((lean_object*)(((size_t)(1) << 1) | 1))}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__2_value;
lean_object* lean_nat_to_int(lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__5_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 17, .m_capacity = 17, .m_length = 16, .m_data = "LDIR.KPItem.glue"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__5 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__5_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__6_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__5_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__6 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__6_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__7_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__6_value),((lean_object*)(((size_t)(1) << 1) | 1))}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__7 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__7_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__8_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 20, .m_capacity = 20, .m_length = 19, .m_data = "LDIR.KPItem.penalty"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__8 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__8_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__9_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__8_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__9 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__9_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__10_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__9_value),((lean_object*)(((size_t)(1) << 1) | 1))}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__10 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__10_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11;
lean_object* l_Nat_reprFast(lean_object*);
lean_object* l_Repr_addAppParen(lean_object*, lean_object*);
uint8_t lean_nat_dec_le(lean_object*, lean_object*);
lean_object* l_Bool_repr___redArg(uint8_t);
uint8_t lean_int_dec_lt(lean_object*, lean_object*);
lean_object* l_Int_repr(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprKPItem___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprKPItem_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprKPItem___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprKPItem = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPItem___closed__0_value;
uint8_t lean_nat_dec_eq(lean_object*, lean_object*);
uint8_t lean_int_dec_eq(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqKPItem_beq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqKPItem_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqKPItem___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqKPItem_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqKPItem___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqKPItem___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqKPItem = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqKPItem___closed__0_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 3, .m_capacity = 3, .m_length = 2, .m_data = "{ "};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__0_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__1_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 9, .m_capacity = 9, .m_length = 8, .m_data = "position"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__1 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__1_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__2_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__1_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__2 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__2_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__3_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)(((size_t)(0) << 1) | 1)),((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__2_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__3 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__3_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__4_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 5, .m_capacity = 5, .m_length = 4, .m_data = " := "};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__4 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__4_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__5_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__4_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__5 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__5_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__6_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*2 + 0, .m_other = 2, .m_tag = 5}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__3_value),((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__5_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__6 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__6_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__7_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__7;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__8_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 2, .m_capacity = 2, .m_length = 1, .m_data = ","};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__8 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__8_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__9_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__8_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__9 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__9_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__10_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 8, .m_capacity = 8, .m_length = 7, .m_data = "fitness"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__10 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__10_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__11_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__10_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__11 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__11_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__12_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__12;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__13_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 13, .m_capacity = 13, .m_length = 12, .m_data = "totalPenalty"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__13 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__13_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__14_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__13_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__14 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__14_value;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__15_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__15;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__16_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 9, .m_capacity = 9, .m_length = 8, .m_data = "previous"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__16 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__16_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__17_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__16_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__17 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__17_value;
static const lean_string_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__18_value = {.m_header = {.m_rc = 0, .m_cs_sz = 0, .m_other = 0, .m_tag = 249}, .m_size = 3, .m_capacity = 3, .m_length = 2, .m_data = " }"};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__18 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__18_value;
lean_object* lean_string_length(lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__19_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__19;
static lean_once_cell_t lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__20_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__20;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__21_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__0_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__21 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__21_value;
static const lean_ctor_object lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__22_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_ctor_object) + sizeof(void*)*1 + 0, .m_other = 1, .m_tag = 3}, .m_objs = {((lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__18_value)}};
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__22 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__22_value;
lean_object* l_Option_repr___at___00Array_repr___at___00Lean_Elab_Structural_instReprRecArgInfo_repr_spec__0_spec__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instReprKPBreak___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instReprKPBreak_repr___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instReprKPBreak = (const lean_object*)&lp_LDIRProofs_LDIR_instReprKPBreak___closed__0_value;
uint8_t lp_mathlib_Option_instBEq_beq___at___00Std_DTreeMap_Internal_Impl_Const_beq___at___00Std_DTreeMap_Const_beq___at___00Std_TreeMap_beq___at___00Mathlib_Tactic_Linarith_Sum_one_spec__0_spec__0_spec__1_spec__4(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqKPBreak_beq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqKPBreak_beq___boxed(lean_object*, lean_object*);
static const lean_closure_object lp_LDIRProofs_LDIR_instBEqKPBreak___closed__0_value = {.m_header = {.m_rc = 0, .m_cs_sz = sizeof(lean_closure_object) + sizeof(void*)*0, .m_other = 0, .m_tag = 245}, .m_fun = (void*)lp_LDIRProofs_LDIR_instBEqKPBreak_beq___boxed, .m_arity = 2, .m_num_fixed = 0, .m_objs = {} };
static const lean_object* lp_LDIRProofs_LDIR_instBEqKPBreak___closed__0 = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqKPBreak___closed__0_value;
LEAN_EXPORT const lean_object* lp_LDIRProofs_LDIR_instBEqKPBreak = (const lean_object*)&lp_LDIRProofs_LDIR_instBEqKPBreak___closed__0_value;
lean_object* l_List_getLast_x3f___redArg(lean_object*);
lean_object* l_List_lengthTR___redArg(lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_validBreakSet(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_validBreakSet___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_demeritsBetween(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_demeritsBetween___boxed(lean_object*, lean_object*, lean_object*);
lean_object* lean_int_add(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_totalDemerits_spec__0(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_totalDemerits_spec__0___boxed(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_totalDemerits(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_totalDemerits___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_kp__findOptimalBreaks(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_kp__findOptimalBreaks___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_lineWidth;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_itemWidth(lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_itemWidth___boxed(lean_object*);
lean_object* lean_nat_add(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_cumWidth_spec__0(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_cumWidth_spec__0___boxed(lean_object*, lean_object*);
lean_object* lean_mk_empty_array_with_capacity(lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_cumWidth___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_cumWidth___closed__0;
lean_object* l___private_Init_Data_List_Impl_0__List_takeTR_go___redArg(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_cumWidth(lean_object*, lean_object*);
lean_object* lean_nat_sub(lean_object*, lean_object*);
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_feasibleBreak(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_feasibleBreak___boxed(lean_object*, lean_object*, lean_object*);
static lean_once_cell_t lp_LDIRProofs_LDIR_demeritsReal___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_LDIRProofs_LDIR_demeritsReal___closed__0;
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_demeritsReal(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_ProofLayoutProperties_0__LDIR_instReprKPItem_repr_match__1_splitter___redArg(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_ProofLayoutProperties_0__LDIR_instReprKPItem_repr_match__1_splitter(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorIdx(lean_object* x_1) {
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
default: 
{
lean_object* x_4; 
x_4 = lean_unsigned_to_nat(2u);
return x_4;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorIdx___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_KPItem_ctorIdx(x_1);
lean_dec_ref(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
switch (lean_obj_tag(x_1)) {
case 0:
{
lean_object* x_3; lean_object* x_4; 
x_3 = lean_ctor_get(x_1, 0);
lean_inc(x_3);
lean_dec_ref(x_1);
x_4 = lean_apply_1(x_2, x_3);
return x_4;
}
case 1:
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; 
x_5 = lean_ctor_get(x_1, 0);
lean_inc(x_5);
x_6 = lean_ctor_get(x_1, 1);
lean_inc(x_6);
x_7 = lean_ctor_get(x_1, 2);
lean_inc(x_7);
lean_dec_ref(x_1);
x_8 = lean_apply_3(x_2, x_5, x_6, x_7);
return x_8;
}
default: 
{
lean_object* x_9; lean_object* x_10; uint8_t x_11; lean_object* x_12; lean_object* x_13; 
x_9 = lean_ctor_get(x_1, 0);
lean_inc(x_9);
x_10 = lean_ctor_get(x_1, 1);
lean_inc(x_10);
x_11 = lean_ctor_get_uint8(x_1, sizeof(void*)*2);
lean_dec_ref(x_1);
x_12 = lean_box(x_11);
x_13 = lean_apply_3(x_2, x_9, x_10, x_12);
return x_13;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorElim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
lean_object* x_6; 
x_6 = lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(x_3, x_5);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_ctorElim___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
lean_object* x_6; 
x_6 = lp_LDIRProofs_LDIR_KPItem_ctorElim(x_1, x_2, x_3, x_4, x_5);
lean_dec(x_2);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_box_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_box_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_glue_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_glue_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_penalty_elim___redArg(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_KPItem_penalty_elim(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_LDIRProofs_LDIR_KPItem_ctorElim___redArg(x_2, x_4);
return x_5;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(2u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(1u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(0u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr(lean_object* x_1, lean_object* x_2) {
_start:
{
switch (lean_obj_tag(x_1)) {
case 0:
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_15; uint8_t x_16; 
x_3 = lean_ctor_get(x_1, 0);
lean_inc(x_3);
if (lean_is_exclusive(x_1)) {
 lean_ctor_release(x_1, 0);
 x_4 = x_1;
} else {
 lean_dec_ref(x_1);
 x_4 = lean_box(0);
}
x_15 = lean_unsigned_to_nat(1024u);
x_16 = lean_nat_dec_le(x_15, x_2);
if (x_16 == 0)
{
lean_object* x_17; 
x_17 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3);
x_5 = x_17;
goto block_14;
}
else
{
lean_object* x_18; 
x_18 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4);
x_5 = x_18;
goto block_14;
}
block_14:
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; uint8_t x_11; lean_object* x_12; lean_object* x_13; 
x_6 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__2));
x_7 = l_Nat_reprFast(x_3);
if (lean_is_scalar(x_4)) {
 x_8 = lean_alloc_ctor(3, 1, 0);
} else {
 x_8 = x_4;
 lean_ctor_set_tag(x_8, 3);
}
lean_ctor_set(x_8, 0, x_7);
x_9 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_9, 0, x_6);
lean_ctor_set(x_9, 1, x_8);
x_10 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_10, 0, x_5);
lean_ctor_set(x_10, 1, x_9);
x_11 = 0;
x_12 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_12, 0, x_10);
lean_ctor_set_uint8(x_12, sizeof(void*)*1, x_11);
x_13 = l_Repr_addAppParen(x_12, x_2);
return x_13;
}
}
case 1:
{
lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; lean_object* x_41; uint8_t x_42; 
x_19 = lean_ctor_get(x_1, 0);
lean_inc(x_19);
x_20 = lean_ctor_get(x_1, 1);
lean_inc(x_20);
x_21 = lean_ctor_get(x_1, 2);
lean_inc(x_21);
lean_dec_ref(x_1);
x_41 = lean_unsigned_to_nat(1024u);
x_42 = lean_nat_dec_le(x_41, x_2);
if (x_42 == 0)
{
lean_object* x_43; 
x_43 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3);
x_22 = x_43;
goto block_40;
}
else
{
lean_object* x_44; 
x_44 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4);
x_22 = x_44;
goto block_40;
}
block_40:
{
lean_object* x_23; lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; lean_object* x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; lean_object* x_35; lean_object* x_36; uint8_t x_37; lean_object* x_38; lean_object* x_39; 
x_23 = lean_box(1);
x_24 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__7));
x_25 = l_Nat_reprFast(x_19);
x_26 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_26, 0, x_25);
x_27 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_27, 0, x_24);
lean_ctor_set(x_27, 1, x_26);
x_28 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_28, 0, x_27);
lean_ctor_set(x_28, 1, x_23);
x_29 = l_Nat_reprFast(x_20);
x_30 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_30, 0, x_29);
x_31 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_31, 0, x_28);
lean_ctor_set(x_31, 1, x_30);
x_32 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_32, 0, x_31);
lean_ctor_set(x_32, 1, x_23);
x_33 = l_Nat_reprFast(x_21);
x_34 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_34, 0, x_33);
x_35 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_35, 0, x_32);
lean_ctor_set(x_35, 1, x_34);
x_36 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_36, 0, x_22);
lean_ctor_set(x_36, 1, x_35);
x_37 = 0;
x_38 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_38, 0, x_36);
lean_ctor_set_uint8(x_38, sizeof(void*)*1, x_37);
x_39 = l_Repr_addAppParen(x_38, x_2);
return x_39;
}
}
default: 
{
lean_object* x_45; lean_object* x_46; uint8_t x_47; lean_object* x_48; lean_object* x_49; lean_object* x_50; lean_object* x_51; lean_object* x_61; lean_object* x_77; uint8_t x_78; 
x_45 = lean_ctor_get(x_1, 0);
lean_inc(x_45);
x_46 = lean_ctor_get(x_1, 1);
lean_inc(x_46);
x_47 = lean_ctor_get_uint8(x_1, sizeof(void*)*2);
lean_dec_ref(x_1);
x_77 = lean_unsigned_to_nat(1024u);
x_78 = lean_nat_dec_le(x_77, x_2);
if (x_78 == 0)
{
lean_object* x_79; 
x_79 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__3);
x_61 = x_79;
goto block_76;
}
else
{
lean_object* x_80; 
x_80 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__4);
x_61 = x_80;
goto block_76;
}
block_60:
{
lean_object* x_52; lean_object* x_53; lean_object* x_54; lean_object* x_55; lean_object* x_56; uint8_t x_57; lean_object* x_58; lean_object* x_59; 
x_52 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_52, 0, x_49);
lean_ctor_set(x_52, 1, x_51);
x_53 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_53, 0, x_52);
lean_ctor_set(x_53, 1, x_50);
x_54 = l_Bool_repr___redArg(x_47);
x_55 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_55, 0, x_53);
lean_ctor_set(x_55, 1, x_54);
x_56 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_56, 0, x_48);
lean_ctor_set(x_56, 1, x_55);
x_57 = 0;
x_58 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_58, 0, x_56);
lean_ctor_set_uint8(x_58, sizeof(void*)*1, x_57);
x_59 = l_Repr_addAppParen(x_58, x_2);
return x_59;
}
block_76:
{
lean_object* x_62; lean_object* x_63; lean_object* x_64; lean_object* x_65; lean_object* x_66; lean_object* x_67; lean_object* x_68; uint8_t x_69; 
x_62 = lean_box(1);
x_63 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__10));
x_64 = l_Nat_reprFast(x_45);
x_65 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_65, 0, x_64);
x_66 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_66, 0, x_63);
lean_ctor_set(x_66, 1, x_65);
x_67 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_67, 0, x_66);
lean_ctor_set(x_67, 1, x_62);
x_68 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11);
x_69 = lean_int_dec_lt(x_46, x_68);
if (x_69 == 0)
{
lean_object* x_70; lean_object* x_71; 
x_70 = l_Int_repr(x_46);
lean_dec(x_46);
x_71 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_71, 0, x_70);
x_48 = x_61;
x_49 = x_67;
x_50 = x_62;
x_51 = x_71;
goto block_60;
}
else
{
lean_object* x_72; lean_object* x_73; lean_object* x_74; lean_object* x_75; 
x_72 = lean_unsigned_to_nat(1024u);
x_73 = l_Int_repr(x_46);
lean_dec(x_46);
x_74 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_74, 0, x_73);
x_75 = l_Repr_addAppParen(x_74, x_72);
x_48 = x_61;
x_49 = x_67;
x_50 = x_62;
x_51 = x_75;
goto block_60;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPItem_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprKPItem_repr(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqKPItem_beq(lean_object* x_1, lean_object* x_2) {
_start:
{
switch (lean_obj_tag(x_1)) {
case 0:
{
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_2, 0);
x_5 = lean_nat_dec_eq(x_3, x_4);
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
lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; uint8_t x_13; 
x_7 = lean_ctor_get(x_1, 0);
x_8 = lean_ctor_get(x_1, 1);
x_9 = lean_ctor_get(x_1, 2);
x_10 = lean_ctor_get(x_2, 0);
x_11 = lean_ctor_get(x_2, 1);
x_12 = lean_ctor_get(x_2, 2);
x_13 = lean_nat_dec_eq(x_7, x_10);
if (x_13 == 0)
{
return x_13;
}
else
{
uint8_t x_14; 
x_14 = lean_nat_dec_eq(x_8, x_11);
if (x_14 == 0)
{
return x_14;
}
else
{
uint8_t x_15; 
x_15 = lean_nat_dec_eq(x_9, x_12);
return x_15;
}
}
}
else
{
uint8_t x_16; 
x_16 = 0;
return x_16;
}
}
default: 
{
if (lean_obj_tag(x_2) == 2)
{
lean_object* x_17; lean_object* x_18; uint8_t x_19; lean_object* x_20; lean_object* x_21; uint8_t x_22; uint8_t x_23; 
x_17 = lean_ctor_get(x_1, 0);
x_18 = lean_ctor_get(x_1, 1);
x_19 = lean_ctor_get_uint8(x_1, sizeof(void*)*2);
x_20 = lean_ctor_get(x_2, 0);
x_21 = lean_ctor_get(x_2, 1);
x_22 = lean_ctor_get_uint8(x_2, sizeof(void*)*2);
x_23 = lean_nat_dec_eq(x_17, x_20);
if (x_23 == 0)
{
return x_23;
}
else
{
uint8_t x_24; 
x_24 = lean_int_dec_eq(x_18, x_21);
if (x_24 == 0)
{
return x_24;
}
else
{
if (x_19 == 0)
{
if (x_22 == 0)
{
return x_24;
}
else
{
return x_19;
}
}
else
{
return x_22;
}
}
}
}
else
{
uint8_t x_25; 
x_25 = 0;
return x_25;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqKPItem_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instBEqKPItem_beq(x_1, x_2);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__7(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(12u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__12(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(11u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__15(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(16u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__19(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__0));
x_2 = lean_string_length(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__20(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__19, &lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__19_once, _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__19);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; uint8_t x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; lean_object* x_23; lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; lean_object* x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; lean_object* x_56; lean_object* x_57; uint8_t x_58; 
x_2 = lean_ctor_get(x_1, 0);
lean_inc(x_2);
x_3 = lean_ctor_get(x_1, 1);
lean_inc(x_3);
x_4 = lean_ctor_get(x_1, 2);
lean_inc(x_4);
x_5 = lean_ctor_get(x_1, 3);
lean_inc(x_5);
lean_dec_ref(x_1);
x_6 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__5));
x_7 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__6));
x_8 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__7, &lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__7_once, _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__7);
x_9 = l_Nat_reprFast(x_2);
x_10 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_10, 0, x_9);
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
x_15 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__9));
x_16 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_16, 0, x_14);
lean_ctor_set(x_16, 1, x_15);
x_17 = lean_box(1);
x_18 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_18, 0, x_16);
lean_ctor_set(x_18, 1, x_17);
x_19 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__11));
x_20 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_20, 0, x_18);
lean_ctor_set(x_20, 1, x_19);
x_21 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_6);
x_22 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__12, &lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__12_once, _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__12);
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
x_30 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__14));
x_31 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_31, 0, x_29);
lean_ctor_set(x_31, 1, x_30);
x_32 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_32, 0, x_31);
lean_ctor_set(x_32, 1, x_6);
x_33 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__15, &lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__15_once, _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__15);
x_56 = lean_unsigned_to_nat(0u);
x_57 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11);
x_58 = lean_int_dec_lt(x_4, x_57);
if (x_58 == 0)
{
lean_object* x_59; lean_object* x_60; 
x_59 = l_Int_repr(x_4);
lean_dec(x_4);
x_60 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_60, 0, x_59);
x_34 = x_60;
goto block_55;
}
else
{
lean_object* x_61; lean_object* x_62; lean_object* x_63; 
x_61 = l_Int_repr(x_4);
lean_dec(x_4);
x_62 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_62, 0, x_61);
x_63 = l_Repr_addAppParen(x_62, x_56);
x_34 = x_63;
goto block_55;
}
block_55:
{
lean_object* x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; lean_object* x_42; lean_object* x_43; lean_object* x_44; lean_object* x_45; lean_object* x_46; lean_object* x_47; lean_object* x_48; lean_object* x_49; lean_object* x_50; lean_object* x_51; lean_object* x_52; lean_object* x_53; lean_object* x_54; 
x_35 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_35, 0, x_33);
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
x_40 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__17));
x_41 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_41, 0, x_39);
lean_ctor_set(x_41, 1, x_40);
x_42 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_42, 0, x_41);
lean_ctor_set(x_42, 1, x_6);
x_43 = lean_unsigned_to_nat(0u);
x_44 = l_Option_repr___at___00Array_repr___at___00Lean_Elab_Structural_instReprRecArgInfo_repr_spec__0_spec__0(x_5, x_43);
x_45 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_45, 0, x_8);
lean_ctor_set(x_45, 1, x_44);
x_46 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_46, 0, x_45);
lean_ctor_set_uint8(x_46, sizeof(void*)*1, x_12);
x_47 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_47, 0, x_42);
lean_ctor_set(x_47, 1, x_46);
x_48 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__20, &lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__20_once, _init_lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__20);
x_49 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__21));
x_50 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_50, 0, x_49);
lean_ctor_set(x_50, 1, x_47);
x_51 = ((lean_object*)(lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg___closed__22));
x_52 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_52, 0, x_50);
lean_ctor_set(x_52, 1, x_51);
x_53 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_53, 0, x_48);
lean_ctor_set(x_53, 1, x_52);
x_54 = lean_alloc_ctor(6, 1, 1);
lean_ctor_set(x_54, 0, x_53);
lean_ctor_set_uint8(x_54, sizeof(void*)*1, x_12);
return x_54;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprKPBreak_repr___redArg(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instReprKPBreak_repr___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_instReprKPBreak_repr(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_instBEqKPBreak_beq(lean_object* x_1, lean_object* x_2) {
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
x_11 = lean_nat_dec_eq(x_3, x_7);
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
x_13 = lean_int_dec_eq(x_5, x_9);
if (x_13 == 0)
{
return x_13;
}
else
{
uint8_t x_14; 
x_14 = lp_mathlib_Option_instBEq_beq___at___00Std_DTreeMap_Internal_Impl_Const_beq___at___00Std_DTreeMap_Const_beq___at___00Std_TreeMap_beq___at___00Mathlib_Tactic_Linarith_Sum_one_spec__0_spec__0_spec__1_spec__4(x_6, x_10);
return x_14;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_instBEqKPBreak_beq___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_instBEqKPBreak_beq(x_1, x_2);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_validBreakSet(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = l_List_getLast_x3f___redArg(x_2);
if (lean_obj_tag(x_3) == 0)
{
lean_object* x_4; lean_object* x_5; uint8_t x_6; 
x_4 = l_List_lengthTR___redArg(x_1);
x_5 = lean_unsigned_to_nat(0u);
x_6 = lean_nat_dec_eq(x_4, x_5);
lean_dec(x_4);
return x_6;
}
else
{
lean_object* x_7; lean_object* x_8; lean_object* x_9; uint8_t x_10; 
x_7 = lean_ctor_get(x_3, 0);
lean_inc(x_7);
lean_dec_ref(x_3);
x_8 = lean_ctor_get(x_7, 0);
lean_inc(x_8);
lean_dec(x_7);
x_9 = l_List_lengthTR___redArg(x_1);
x_10 = lean_nat_dec_eq(x_8, x_9);
lean_dec(x_9);
lean_dec(x_8);
return x_10;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_validBreakSet___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lp_LDIRProofs_LDIR_validBreakSet(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile___redArg(lean_object* x_1) {
_start:
{
lean_inc(x_1);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile___redArg___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_recompile___redArg(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_inc(x_2);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_recompile___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_recompile(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_demeritsBetween(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
x_4 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_demeritsBetween___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
x_4 = lp_LDIRProofs_LDIR_demeritsBetween(x_1, x_2, x_3);
lean_dec(x_3);
lean_dec(x_2);
lean_dec(x_1);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_totalDemerits_spec__0(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
if (lean_obj_tag(x_3) == 0)
{
return x_2;
}
else
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_4 = lean_ctor_get(x_3, 0);
x_5 = lean_ctor_get(x_3, 1);
x_6 = lean_ctor_get(x_4, 0);
x_7 = lean_ctor_get(x_4, 3);
x_8 = lp_LDIRProofs_LDIR_demeritsBetween(x_1, x_6, x_7);
x_9 = lean_int_add(x_2, x_8);
lean_dec(x_8);
lean_dec(x_2);
x_2 = x_9;
x_3 = x_5;
goto _start;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_totalDemerits_spec__0___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
x_4 = lp_LDIRProofs_List_foldl___at___00LDIR_totalDemerits_spec__0(x_1, x_2, x_3);
lean_dec(x_3);
lean_dec(x_1);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_totalDemerits(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; 
x_3 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11);
x_4 = lp_LDIRProofs_List_foldl___at___00LDIR_totalDemerits_spec__0(x_1, x_3, x_2);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_totalDemerits___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_LDIR_totalDemerits(x_1, x_2);
lean_dec(x_2);
lean_dec(x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_kp__findOptimalBreaks(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lean_box(0);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_kp__findOptimalBreaks___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_kp__findOptimalBreaks(x_1);
lean_dec(x_1);
return x_2;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_lineWidth(void) {
_start:
{
lean_object* x_1; 
x_1 = lean_unsigned_to_nat(324u);
return x_1;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_itemWidth(lean_object* x_1) {
_start:
{
if (lean_obj_tag(x_1) == 1)
{
lean_object* x_2; 
x_2 = lean_ctor_get(x_1, 2);
lean_inc(x_2);
return x_2;
}
else
{
lean_object* x_3; 
x_3 = lean_ctor_get(x_1, 0);
lean_inc(x_3);
return x_3;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_itemWidth___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_LDIRProofs_LDIR_itemWidth(x_1);
lean_dec_ref(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_cumWidth_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
return x_1;
}
else
{
lean_object* x_3; 
x_3 = lean_ctor_get(x_2, 0);
if (lean_obj_tag(x_3) == 1)
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; 
x_4 = lean_ctor_get(x_2, 1);
x_5 = lean_ctor_get(x_3, 2);
x_6 = lean_nat_add(x_1, x_5);
lean_dec(x_1);
x_1 = x_6;
x_2 = x_4;
goto _start;
}
else
{
lean_object* x_8; lean_object* x_9; lean_object* x_10; 
x_8 = lean_ctor_get(x_2, 1);
x_9 = lean_ctor_get(x_3, 0);
x_10 = lean_nat_add(x_1, x_9);
lean_dec(x_1);
x_1 = x_10;
x_2 = x_8;
goto _start;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_List_foldl___at___00LDIR_cumWidth_spec__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_LDIRProofs_List_foldl___at___00LDIR_cumWidth_spec__0(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_cumWidth___closed__0(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(0u);
x_2 = lean_mk_empty_array_with_capacity(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_cumWidth(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; 
x_3 = lean_unsigned_to_nat(0u);
x_4 = lean_obj_once(&lp_LDIRProofs_LDIR_cumWidth___closed__0, &lp_LDIRProofs_LDIR_cumWidth___closed__0_once, _init_lp_LDIRProofs_LDIR_cumWidth___closed__0);
lean_inc(x_1);
x_5 = l___private_Init_Data_List_Impl_0__List_takeTR_go___redArg(x_1, x_1, x_2, x_4);
lean_dec(x_1);
x_6 = lp_LDIRProofs_List_foldl___at___00LDIR_cumWidth_spec__0(x_3, x_5);
lean_dec(x_5);
return x_6;
}
}
LEAN_EXPORT uint8_t lp_LDIRProofs_LDIR_feasibleBreak(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
if (lean_obj_tag(x_3) == 0)
{
lean_object* x_11; 
x_11 = lean_unsigned_to_nat(0u);
x_4 = x_11;
goto block_10;
}
else
{
lean_object* x_12; 
x_12 = lean_ctor_get(x_3, 0);
lean_inc(x_12);
lean_dec_ref(x_3);
x_4 = x_12;
goto block_10;
}
block_10:
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; uint8_t x_9; 
lean_inc(x_1);
x_5 = lp_LDIRProofs_LDIR_cumWidth(x_1, x_2);
x_6 = lp_LDIRProofs_LDIR_cumWidth(x_1, x_4);
x_7 = lean_nat_sub(x_5, x_6);
lean_dec(x_6);
lean_dec(x_5);
x_8 = lean_unsigned_to_nat(324u);
x_9 = lean_nat_dec_le(x_7, x_8);
lean_dec(x_7);
return x_9;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_feasibleBreak___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
uint8_t x_4; lean_object* x_5; 
x_4 = lp_LDIRProofs_LDIR_feasibleBreak(x_1, x_2, x_3);
x_5 = lean_box(x_4);
return x_5;
}
}
static lean_object* _init_lp_LDIRProofs_LDIR_demeritsReal___closed__0(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(10000u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs_LDIR_demeritsReal(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
uint8_t x_4; 
x_4 = lp_LDIRProofs_LDIR_feasibleBreak(x_1, x_2, x_3);
if (x_4 == 0)
{
lean_object* x_5; 
x_5 = lean_obj_once(&lp_LDIRProofs_LDIR_demeritsReal___closed__0, &lp_LDIRProofs_LDIR_demeritsReal___closed__0_once, _init_lp_LDIRProofs_LDIR_demeritsReal___closed__0);
return x_5;
}
else
{
lean_object* x_6; 
x_6 = lean_obj_once(&lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11, &lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11_once, _init_lp_LDIRProofs_LDIR_instReprKPItem_repr___closed__11);
return x_6;
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_ProofLayoutProperties_0__LDIR_instReprKPItem_repr_match__1_splitter___redArg(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
switch (lean_obj_tag(x_1)) {
case 0:
{
lean_object* x_5; lean_object* x_6; 
lean_dec(x_4);
lean_dec(x_3);
x_5 = lean_ctor_get(x_1, 0);
lean_inc(x_5);
lean_dec_ref(x_1);
x_6 = lean_apply_1(x_2, x_5);
return x_6;
}
case 1:
{
lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; 
lean_dec(x_4);
lean_dec(x_2);
x_7 = lean_ctor_get(x_1, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_1, 1);
lean_inc(x_8);
x_9 = lean_ctor_get(x_1, 2);
lean_inc(x_9);
lean_dec_ref(x_1);
x_10 = lean_apply_3(x_3, x_7, x_8, x_9);
return x_10;
}
default: 
{
lean_object* x_11; lean_object* x_12; uint8_t x_13; lean_object* x_14; lean_object* x_15; 
lean_dec(x_3);
lean_dec(x_2);
x_11 = lean_ctor_get(x_1, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_1, 1);
lean_inc(x_12);
x_13 = lean_ctor_get_uint8(x_1, sizeof(void*)*2);
lean_dec_ref(x_1);
x_14 = lean_box(x_13);
x_15 = lean_apply_3(x_4, x_11, x_12, x_14);
return x_15;
}
}
}
}
LEAN_EXPORT lean_object* lp_LDIRProofs___private_LDIRProofs_ProofLayoutProperties_0__LDIR_instReprKPItem_repr_match__1_splitter(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4, lean_object* x_5) {
_start:
{
switch (lean_obj_tag(x_2)) {
case 0:
{
lean_object* x_6; lean_object* x_7; 
lean_dec(x_5);
lean_dec(x_4);
x_6 = lean_ctor_get(x_2, 0);
lean_inc(x_6);
lean_dec_ref(x_2);
x_7 = lean_apply_1(x_3, x_6);
return x_7;
}
case 1:
{
lean_object* x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; 
lean_dec(x_5);
lean_dec(x_3);
x_8 = lean_ctor_get(x_2, 0);
lean_inc(x_8);
x_9 = lean_ctor_get(x_2, 1);
lean_inc(x_9);
x_10 = lean_ctor_get(x_2, 2);
lean_inc(x_10);
lean_dec_ref(x_2);
x_11 = lean_apply_3(x_4, x_8, x_9, x_10);
return x_11;
}
default: 
{
lean_object* x_12; lean_object* x_13; uint8_t x_14; lean_object* x_15; lean_object* x_16; 
lean_dec(x_4);
lean_dec(x_3);
x_12 = lean_ctor_get(x_2, 0);
lean_inc(x_12);
x_13 = lean_ctor_get(x_2, 1);
lean_inc(x_13);
x_14 = lean_ctor_get_uint8(x_2, sizeof(void*)*2);
lean_dec_ref(x_2);
x_15 = lean_box(x_14);
x_16 = lean_apply_3(x_5, x_12, x_13, x_15);
return x_16;
}
}
}
}
lean_object* initialize_Init(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_Data_List_Basic(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_Data_List_Lemmas(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_Tactic(uint8_t builtin);
lean_object* initialize_LDIRProofs_LDIRProofs_proof__ir__wellformedness(uint8_t builtin);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_LDIRProofs_LDIRProofs_ProofLayoutProperties(uint8_t builtin) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_Data_List_Basic(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_Data_List_Lemmas(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_Tactic(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_LDIRProofs_LDIRProofs_proof__ir__wellformedness(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
lp_LDIRProofs_LDIR_lineWidth = _init_lp_LDIRProofs_LDIR_lineWidth();
lean_mark_persistent(lp_LDIRProofs_LDIR_lineWidth);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
