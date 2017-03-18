// Auto generated {{date}}

use neovim::*;
use rpc::*;

{% for typename in exttypes %}
pub struct {{ typename }} {
    code_data: Value,
}

impl {{ typename }} {
    pub fn new(code_data: Value) -> {{ typename }} {
        {{ typename }} {
            code_data: code_data,
        }
    }

    {% for f in functions if f.ext and f.name.startswith(typename.lower()) %}
    pub fn {{f.name|replace(typename.lower() + '_', '')}}(&self, neovim: &mut Neovim, {{f.argstring}}) -> Result<{{f.return_type.native_type_ret}}, CallError> {
        neovim.session.call("{{f.name}}",
                          &call_args![self.code_data.clone()
                          {% if f.parameters|count > 0 %}
                          , {{ f.parameters|map(attribute = "name")|join(", ") }}
                          {% endif %}
                          ])
                    .map(map_result)
                    .map_err(map_generic_error)
    }
    {% endfor %}
}

{% endfor %}

{% for typename in exttypes %}
impl FromVal<Value> for {{ typename }} {
    fn from_val(val: Value) -> Self {
        {{ typename }}::new(val)
    }
}

impl <'a> IntoVal<Value> for &'a {{typename}} {
    fn into_val(self) -> Value {
        self.code_data.clone()
    }
}
{% endfor %}

pub trait NeovimApi {
    {% for f in functions if not f.ext %}
    fn {{f.name|replace('vim_', '')}}(&mut self, {{f.argstring}}) -> Result<{{f.return_type.native_type_ret}}, CallError>;
    {% endfor %}
}

impl NeovimApi for Neovim {
    {% for f in functions if not f.ext %}
    fn {{f.name|replace('vim_', '')}}(&mut self, {{f.argstring}}) -> Result<{{f.return_type.native_type_ret}}, CallError> {
        self.session.call("{{f.name}}",
                          &call_args![{{ f.parameters|map(attribute = "name")|join(", ") }}])
                    .map(map_result)
                    .map_err(map_generic_error)
    }

    {% endfor %}
}
