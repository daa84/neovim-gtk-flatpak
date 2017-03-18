// Copyright 2017, Reizner Evgeniy <razrfalcon@gmail.com>.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

extern crate pangocairo_sys as ffi;
extern crate cairo;
extern crate pango;
extern crate glib;

use glib::translate::*;

//=========================================================================
// PangoCairoFcFontMap
//=========================================================================
// pub fn pango_cairo_fc_font_map_get_type() -> GType;


//=========================================================================
// PangoCairoFont
//=========================================================================
// pub fn pango_cairo_font_get_type() -> GType;
// pub fn pango_cairo_font_get_scaled_font(font: *mut PangoCairoFont) -> *mut cairo::cairo_scaled_font_t;


//=========================================================================
// PangoCairoFontMap
//=========================================================================
// pub fn pango_cairo_font_map_get_type() -> GType;

pub fn font_map_get_default() -> pango::FontMap {
    unsafe { from_glib_full(ffi::pango_cairo_font_map_get_default()) }
}

pub fn font_map_new() -> pango::FontMap {
    unsafe { from_glib_full(ffi::pango_cairo_font_map_new()) }
}

pub fn font_map_new_for_font_type(fonttype: cairo::FontType) -> pango::FontMap {
    unsafe { from_glib_full(ffi::pango_cairo_font_map_new_for_font_type(fonttype)) }
}

// pub fn pango_cairo_font_map_create_context(fontmap: *mut PangoCairoFontMap) -> *mut pango::PangoContext;
// pub fn pango_cairo_font_map_get_font_type(fontmap: *mut PangoCairoFontMap) -> cairo::enums::FontType;
// pub fn pango_cairo_font_map_get_resolution(fontmap: *mut PangoCairoFontMap) -> c_double;
// pub fn pango_cairo_font_map_set_default(fontmap: *mut PangoCairoFontMap);
// pub fn pango_cairo_font_map_set_resolution(fontmap: *mut PangoCairoFontMap, dpi: c_double);


//=========================================================================
// Other functions
//=========================================================================
// pub fn pango_cairo_context_get_font_options(context: *mut pango::PangoContext)
//     -> *const cairo::cairo_font_options_t;

pub fn context_get_resolution(context: &pango::Context) -> f64 {
    unsafe { ffi::pango_cairo_context_get_resolution(context.to_glib_none().0) }
}

// pub fn pango_cairo_context_get_shape_renderer(context: *mut pango::PangoContext, data: *mut gpointer)
//     -> PangoCairoShapeRendererFunc;

// pub fn pango_cairo_context_set_font_options(context: *mut pango::PangoContext,
//                                             options: *const cairo::cairo_font_options_t);

pub fn context_set_resolution(context: &pango::Context, dpi: f64) {
    unsafe { ffi::pango_cairo_context_set_resolution(context.to_glib_none().0, dpi); }
}

// pub fn pango_cairo_context_set_shape_renderer(context: *mut pango::PangoContext,
//                                               func: PangoCairoShapeRendererFunc, data: gpointer,
//                                               dnotify: glib::GDestroyNotify);

pub fn create_context(cr: &cairo::Context) -> pango::Context {
    unsafe { from_glib_full(ffi::pango_cairo_create_context(cr.to_glib_none().0)) }
}

pub fn create_layout(cr: &cairo::Context) -> pango::Layout {
    unsafe { from_glib_full(ffi::pango_cairo_create_layout(cr.to_glib_none().0)) }
}

pub fn error_underline_path(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64) {
    unsafe { ffi::pango_cairo_error_underline_path(cr.to_glib_none().0, x, y, width, height); };
}

// pub fn pango_cairo_glyph_string_path(cr: *mut cairo::cairo_t, font: *mut pango::PangoFont,
//                                      glyphs: *mut pango::PangoGlyphString);

pub fn layout_line_path(cr: &cairo::Context, line: &pango::LayoutLine) {
    unsafe { ffi::pango_cairo_layout_line_path(cr.to_glib_none().0, line.to_glib_none().0); };
}

pub fn layout_path(cr: &cairo::Context, layout: &pango::Layout) {
    unsafe { ffi::pango_cairo_layout_path(cr.to_glib_none().0, layout.to_glib_none().0); };
}

pub fn show_error_underline(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64) {
    unsafe { ffi::pango_cairo_show_error_underline(cr.to_glib_none().0, x, y, width, height); };
}

// pub fn pango_cairo_show_glyph_item(cr: &cairo::Context, text: &str, glyph_item: &pango::GlyphItem) {
//     unsafe { ffi::pango_cairo_show_glyph_item(cr.to_glib_none().0, text.to_glib_none().0,
//                                               glyph_item.to_glib_none().0); };
// }

// pub fn show_glyph_string(cr: &cairo::Context, font: &pango::Font, glyphs: &pango::GlyphString) {
//     unsafe { ffi::pango_cairo_show_glyph_string(cr.to_glib_none().0, font.to_glib_none().0,
//                                                 glyphs.to_glib_none().0); };
// }

pub fn show_layout(cr: &cairo::Context, layout: &pango::Layout) {
    unsafe { ffi::pango_cairo_show_layout(cr.to_glib_none().0, layout.to_glib_none().0); };
}

pub fn show_layout_line(cr: &cairo::Context, line: &pango::LayoutLine) {
    unsafe { ffi::pango_cairo_show_layout_line(cr.to_glib_none().0, line.to_glib_none().0); };
}

pub fn update_context(cr: &cairo::Context, context: &pango::Context) {
    unsafe { ffi::pango_cairo_update_context(cr.to_glib_none().0, context.to_glib_none().0); };
}

pub fn update_layout(cr: &cairo::Context, layout: &pango::Layout) {
    unsafe { ffi::pango_cairo_update_layout(cr.to_glib_none().0, layout.to_glib_none().0); };
}
