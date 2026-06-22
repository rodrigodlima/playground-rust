# Rust em Foco — O que é diferente de outras linguagens

> Companheiro de `EXPLICACAO.md`. Aqui o foco **não é o algoritmo**, e sim **cada
> recurso da linguagem Rust** que aparece em `segment_tree.rs`, `metrics.rs` e no
> exemplo `web_server_metrics.rs` — explicando o que é estranho/novo para quem vem
> de Java, C#, Python, JavaScript, C++ etc.

Código de referência (backup):

```rust
pub struct SegmentTree<T, F>
where
    T: Clone,
    F: Fn(&T, &T) -> T,
{
    n: usize,
    tree: Vec<T>,
    identity: T,
    combine: F,
}
```

---

## Índice

1. [`self`, `&self`, `&mut self` — ownership do método](#1-self-self-mut-self)
2. [`fn` vs `Fn` (e `FnMut`/`FnOnce`)](#2-fn-vs-fn)
3. [Generics com `where` e trait bounds](#3-generics-e-trait-bounds)
4. [`Clone` e por que `.clone()` aparece tanto](#4-clone)
5. [Ownership, Borrowing e Referências (`&`, `&mut`, `*`)](#5-ownership-borrowing)
6. [`Vec<T>`, slices `&[T]` e `usize`](#6-vec-slices-usize)
7. [`Option<T>` em vez de `null`](#7-option)
8. [`Self` vs `self` — maiúsculo vs minúsculo](#8-self-maiusculo)
9. [Sem `return`, sem `;` — expressões](#9-expressoes)
10. [`impl` separado do `struct`](#10-impl)
11. [`pub`, módulos e `crate::`](#11-pub-modulos)
12. [`derive`, `Default` e atributos `#[...]`](#12-derive-default)
13. [`assert!`, `panic!` e o `!` (macros)](#13-macros)
14. [Tuplas, destructuring e o `&` em `for`](#14-tuplas-destructuring)
15. [Casts com `as` (sem conversão implícita)](#15-casts)
16. [`let mut` — imutável por padrão](#16-let-mut)

---

## 1. `self`, `&self`, `&mut self`

A maior diferença para Java/C#/Python. O **primeiro parâmetro** de um método declara
**como ele acessa o objeto**. Não existe `this` mágico — você escolhe a forma de acesso.

```rust
pub fn query(&self, l: usize, r: usize) -> T   // só lê
pub fn update(&mut self, idx: usize, val: T)   // lê e modifica
pub fn new(...) -> Self                          // não recebe self (é "static")
```

| Forma | Significa | Equivalente mental | Quem usa no código |
|-------|-----------|--------------------|--------------------|
| `&self` | empresta o objeto **só leitura** | `const` method em C++ | `query`, `len`, `is_empty`, `raw_slot` |
| `&mut self` | empresta o objeto para **modificar** | método normal mutável | `update`, `build`, `record` |
| `self` | **consome** o objeto (toma posse, move) | objeto deixa de existir pro chamador | — (não usado aqui) |
| sem `self` | função associada (estática) | `static` method | `new`, `from_data` |

**Por que importa (não tem equivalente em Java/Python):**

- `&self` ⇒ o compilador **garante** que o método não altera o objeto. Pode chamar em
  paralelo, pode ter vários `&self` ao mesmo tempo.
- `&mut self` ⇒ **acesso exclusivo**. Enquanto um `&mut` existe, ninguém mais pode ler
  nem escrever. Isso é o que elimina data races em tempo de compilação.
- `self` (sem `&`) ⇒ o objeto é **movido** para dentro do método; depois disso o
  chamador não pode mais usá-lo. Útil em builders/conversões (`into_x`).

Em Java/Python todo método recebe `this`/`self` por referência mutável implícita —
você nunca escolhe. Em Rust a escolha é **parte da assinatura** e é verificada.

---

## 2. `fn` vs `Fn`

Aparecem os dois e parecem iguais — não são.

```rust
merge: fn(&T, &T) -> T          // src antigo: ponteiro de função
F: Fn(&T, &T) -> T              // backup: trait de closure
```

### `fn` (minúsculo) — **ponteiro de função**
- Um tipo concreto: endereço de uma função.
- Não captura nada do ambiente. Só serve para funções "puras" como `fn add_u64(a,b)`.
- Tamanho fixo (um ponteiro). Por isso `metrics.rs` escreve o tipo por extenso:
  ```rust
  count_tree: SegmentTree<u64, fn(&u64, &u64) -> u64>,
  ```

### `Fn` (maiúsculo) — **trait** implementada por closures
- Não é um tipo, é um **comportamento** ("isto pode ser chamado como função").
- Toda closure `|a, b| ...` implementa uma das três traits abaixo.
- Permite **capturar variáveis** do ambiente (uma `fn` não consegue).
- Usado como bound genérico: `F: Fn(&T, &T) -> T` ⇒ "F é qualquer coisa chamável".

As três traits de chamável:

| Trait | Pode... | Self do call |
|-------|---------|--------------|
| `Fn` | chamar várias vezes, só lê o capturado | `&self` |
| `FnMut` | chamar várias vezes, **modifica** o capturado | `&mut self` |
| `FnOnce` | chamar **uma vez** (consome o capturado) | `self` |

Toda `fn` ponteiro também implementa `Fn` — por isso `add_u64` (uma `fn`) pode ser
passado onde se espera `F: Fn(...)`. O contrário não vale: uma closure que captura
**não** vira `fn`.

**Resumo:** use `fn` quando precisar de tipo concreto/simples sem captura; use o
bound `Fn`/`FnMut`/`FnOnce` quando quiser aceitar closures genéricas.

---

## 3. Generics e trait bounds

```rust
pub struct SegmentTree<T, F>
where
    T: Clone,
    F: Fn(&T, &T) -> T,
{ ... }
```

- `<T, F>` ⇒ generics, como `<T>` em Java/C#. Mas em Rust são **monomorfizados**:
  o compilador gera uma cópia especializada por tipo concreto (igual templates de C++,
  **sem** boxing/reflection como Java).
- `where T: Clone` ⇒ **trait bound**: "T só serve se implementar a trait `Clone`".
  Equivale a `<T extends Comparable>` em Java, mas baseado em **traits** (interfaces),
  não herança.
- `F: Fn(&T,&T)->T` ⇒ bound sobre comportamento de função (ver seção 2).

Diferença-chave: sem o bound `T: Clone`, chamar `.clone()` em um `T` **não compila**.
O compilador exige que você declare toda capacidade que usa.

---

## 4. `Clone`

```rust
let tree = vec![identity.clone(); 4 * n];
self.tree[node] = data[start].clone();
return self.identity.clone();
```

- Rust **não copia objetos implicitamente** (diferente de C++ copy constructor que
  roda sozinho, ou de Java onde tudo é referência).
- `.clone()` é uma **cópia explícita e profunda**, só disponível se `T: Clone`.
- Aparece muito porque mover um valor para dentro do `Vec` o **consumiria**; quando se
  quer manter o original (o `identity`), clona-se.
- `merge_children` clona os filhos antes de combinar:
  ```rust
  let left  = self.tree[2*node].clone();
  let right = self.tree[2*node+1].clone();
  self.tree[node] = (self.combine)(&left, &right);
  ```
  **Por quê?** Não dá para emprestar `&self.tree[a]` e `&self.tree[b]` e ao mesmo tempo
  escrever em `self.tree[node]` — seriam borrows simultâneos (imutável + mutável) do
  mesmo `Vec`, o que o borrow checker proíbe. Clonar resolve o conflito. (Comentário no
  código diz exatamente isso.)

`Copy` vs `Clone`: tipos pequenos (`u64`, `f64`, `usize`, `bool`) são `Copy` — copiados
de graça na atribuição. `Clone` é a versão explícita e geral.

---

## 5. Ownership, Borrowing e Referências (`&`, `&mut`, `*`)

O coração do Rust. Não existe em Java/Python/Go.

- **Ownership:** cada valor tem **um dono**. Quando o dono sai de escopo, o valor é
  liberado (sem GC, sem `free` manual).
- **Move:** atribuir/passar um valor não-`Copy` **transfere a posse**. O original fica
  inválido. (Por isso clona-se quando precisa dos dois.)
- **Borrow:** `&x` empresta sem transferir posse. `&mut x` empresta para modificar.

Regra que o compilador força (a "regra do borrow"):
> A qualquer momento: **ou** vários `&` (leitura), **ou** exatamente um `&mut`
> (escrita). Nunca os dois juntos.

No código:

```rust
fn add_f64(a: &f64, b: &f64) -> f64 { *a + *b }
```
- `a: &f64` ⇒ recebe **referência** a um `f64` (não copia, não toma posse).
- `*a` ⇒ **deref**: acessa o valor apontado. Precisa do `*` porque `a` é `&f64`, não `f64`.

```rust
let s = &mut self.raw[slot];   // empréstimo mutável de um slot
s.count += 1;
...
// captura valores ANTES de soltar o borrow:
let (count, ...) = (s.count, ...);
self.count_tree.update(slot, count);  // agora pode emprestar self de novo
```
Comentário do código: *"Capture values before releasing the mutable borrow on raw."*
Enquanto `s` (= `&mut self.raw[..]`) existe, não se pode tocar em outras partes de
`self`. Copiam-se os números primitivos (`Copy`) e o borrow some.

`&left` em `(self.combine)(&left, &right)` ⇒ passa referências para a closure, que
espera `Fn(&T, &T)`.

---

## 6. `Vec<T>`, slices `&[T]` e `usize`

- `Vec<T>` ⇒ array dinâmico dono dos dados (como `ArrayList`/`vector`/`list`).
  `vec![valor; n]` cria `n` cópias. `4 * n` aqui dimensiona a árvore.
- `&[T]` ⇒ **slice**: uma "janela" emprestada sobre um array/Vec (ponteiro + tamanho).
  `from_data(data: &[T], ...)` aceita slice ⇒ funciona com `Vec`, array fixo, etc., sem
  copiar. Não existe equivalente direto em Java (lá passaria-se a lista inteira).
- `usize` ⇒ inteiro sem sinal do tamanho do ponteiro (64 bits em máquina 64-bit). É o
  **tipo obrigatório para índices** (`vec[i]` exige `usize`). Não dá para indexar com
  `i32`/`u64` sem cast. Por isso `total_requests as f64` e `i % 5 as f64` aparecem.

---

## 7. `Option<T>` em vez de `null`

```rust
pub min_response_ms: Option<f64>,   // None quando não houve requests
...
(Some(min_ms), Some(max_ms), Some(sum_ms / total_requests as f64))
...
(None, None, None)
```

- Rust **não tem `null`**. Ausência de valor é o enum `Option<T>` com variantes
  `Some(x)` ou `None`.
- O compilador **obriga** a tratar o `None` antes de usar o valor ⇒ acaba o
  `NullPointerException`.
- No exemplo, extrai-se com `.unwrap()`:
  ```rust
  s.avg_response_ms.unwrap()
  ```
  `unwrap()` = "tenho certeza que é `Some`, me dá o valor — ou dê **panic** se for
  `None`". Usado aqui só depois de checar `if s.total_requests > 0`.

---

## 8. `Self` (maiúsculo) vs `self` (minúsculo)

```rust
pub fn new(...) -> Self {
    SegmentTree { n, tree, identity, combine }
}
```

- `Self` (maiúsculo) ⇒ **o tipo** sendo implementado (`SegmentTree<T, F>`). Atalho para
  não repetir o nome completo no retorno.
- `self` (minúsculo) ⇒ **a instância** (ver seção 1).

Construtor: Rust não tem `constructor` nem `new` como palavra-chave. `new` é só uma
**convenção de nome** para função associada que retorna `Self`. Note também o
**field init shorthand**: `SegmentTree { n, tree, ... }` em vez de `n: n, tree: tree`
quando a variável tem o mesmo nome do campo.

---

## 9. Sem `return`, sem `;` — expressões

```rust
pub fn is_empty(&self) -> bool {
    self.n == 0          // sem ; e sem return = valor retornado
}

let (min, max, avg) = if total_requests > 0 {
    (Some(min_ms), ...)  // if é EXPRESSÃO, retorna valor
} else {
    (None, None, None)
};
```

- A **última expressão sem `;`** de um bloco é o valor do bloco. `return` é opcional
  (usado só para sair cedo, como em `query_inner`).
- `if/else`, `match`, blocos `{}` são **expressões** que produzem valor — dá para
  atribuir direto. Em Java/C isso seria operador ternário ou variável + atribuições.
- O `;` transforma expressão em **statement** (descarta o valor). Esquecer ou colocar
  `;` muda o significado.

---

## 10. `impl` separado do `struct`

```rust
pub struct SegmentTree<T, F> { ... }   // só os DADOS

impl<T, F> SegmentTree<T, F>           // os MÉTODOS, em bloco separado
where T: Clone, F: Fn(&T,&T)->T
{
    pub fn new(...) {}
    pub fn query(...) {}
}
```

- Dados e comportamento ficam **separados**. Diferente de Java/C#/Python onde campos e
  métodos vivem dentro da mesma `class { }`.
- Pode haver **vários** blocos `impl` para o mesmo tipo, inclusive em arquivos
  diferentes, e blocos `impl Trait for Tipo` para implementar traits.
- Os bounds (`where`) precisam ser repetidos no `impl`.

---

## 11. `pub`, módulos e `crate::`

```rust
pub mod segment_tree;        // lib.rs declara o módulo
pub struct SegmentTree ...   // pub = visível fora do módulo
fn build(...)                // sem pub = privado ao módulo
use crate::SegmentTree;      // metrics.rs importa do crate atual
use segment_tree::MetricsCollector;  // exemplo importa do crate (lib)
```

- Tudo é **privado por padrão**. `pub` expõe. Mais restritivo que o `public` default de
  Python/JS.
- `mod` define módulos; o nome do arquivo vira o módulo (`segment_tree.rs` ⇒ `mod
  segment_tree`).
- `crate::` ⇒ raiz do **crate** (o pacote/compilação atual), como um caminho absoluto
  interno. `super::` seria o módulo pai.
- Campos de struct também têm visibilidade: `n`, `tree` são privados; em `SlotMetrics`
  os campos são `pub` ⇒ acessíveis de fora.

---

## 12. `derive`, `Default` e atributos `#[...]`

```rust
#[derive(Debug, Clone)]
pub struct SlotMetrics { ... }
```

- `#[...]` ⇒ **atributo** (metadado para o compilador), parecido com anotações Java mas
  processados em compile-time.
- `#[derive(...)]` ⇒ o compilador **gera a implementação** da trait automaticamente:
  - `Debug` ⇒ permite `{:?}` no `println!` (impressão para debug).
  - `Clone` ⇒ gera o `.clone()` campo a campo.
- `Default` é implementado **à mão** aqui porque o padrão de `f64` é `0.0`, mas quer-se
  `INFINITY`/`NEG_INFINITY`:
  ```rust
  impl Default for SlotMetrics {
      fn default() -> Self { SlotMetrics { min_ms: f64::INFINITY, ... } }
  }
  ```
  `vec![SlotMetrics::default(); n]` usa isso para inicializar todos os slots.

---

## 13. `assert!`, `panic!` e o `!` (macros)

```rust
assert!(n > 0, "segment tree size must be > 0");
assert!(idx < self.n, "index {idx} out of bounds (size: {})", self.n);
```

- O `!` indica **macro**, não função. `assert!`, `vec!`, `println!`, `print!` são
  macros — expandem em código em compile-time e aceitam número variável de argumentos /
  sintaxe especial (impossível para função normal).
- `assert!` ⇒ se a condição for falsa, **panic** (aborta a thread com a mensagem).
  É a forma de "exceção" para erros de programação. (Erros recuperáveis usam
  `Result<T, E>`, não presente neste código.)
- **String interpolation:** `"{idx}"` injeta a variável `idx` direto (Rust moderno),
  e `{}` consome o próximo argumento posicional (`self.n`). `{:>5}`, `{:4.1}`, `{:02}`,
  `{:?}` são **format specs** (largura, casas decimais, zero-padding, debug).

---

## 14. Tuplas, destructuring e o `&` em `for`

```rust
let (count, error_count, sum_ms, min_ms, max_ms) =
    (s.count, s.error_count, s.sum_ms, s.min_ms, s.max_ms);
```
- **Tupla** = grupo fixo de valores de tipos possivelmente diferentes. Permite
  retornar/atribuir vários valores de uma vez. `let (a, b) = ...` é **destructuring**.

```rust
let hourly_data: &[(usize, u64, f64, u64)] = &[ (0, 50, 80.0, 1), ... ];

for &(hour, count, base_ms, errors) in hourly_data { ... }
```
- Slice de tuplas. O `&(hour, count, ...)` no `for` é **pattern matching** que faz
  destructuring **e** deref ao mesmo tempo: cada item iterado é `&(...)`, e o `&` no
  padrão "desembrulha" a referência copiando os campos (todos `Copy`).
- Sem o `&` no padrão, `hour` etc. seriam referências — teria que usar `*`.

---

## 15. Casts com `as` (sem conversão implícita)

```rust
sum_ms / total_requests as f64
error_count as f64 / total_requests as f64
(i % 5) as f64
0..24usize
```

- Rust **não converte números automaticamente** (diferente de C/Java que promovem
  `int`→`double` sozinhos). Misturar `u64` e `f64` sem `as` **não compila**.
- `x as f64` ⇒ cast explícito. Necessário antes de dividir contagem (`u64`) para obter
  média (`f64`).
- `24usize` / `0u64` / `0.0f64` ⇒ **sufixo de tipo** no literal, para fixar o tipo
  quando o inferidor precisa de ajuda.

---

## 16. `let mut` — imutável por padrão

```rust
let mut collector = MetricsCollector::new(24);   // mut: vai ser modificado
let mut st = SegmentTree { ... };                // mut: build() altera depois
let tree = vec![...];                            // sem mut: nunca reatribuído
```

- Variáveis são **imutáveis por padrão**. Para modificar/reatribuir, precisa de `mut`.
  Oposto de Java/Python/JS onde tudo é mutável a menos que use `final`/`const`.
- `mut` na variável e `&mut` no empréstimo são coisas distintas: `mut` = "esta ligação
  pode mudar"; `&mut` = "esta referência permite mudar o alvo".
- Chamar `collector.record(...)` exige `let mut collector` porque `record` recebe
  `&mut self`.

---

## Resumo — tabela rápida

| Símbolo / palavra | É | Diferente de outras linguagens porque... |
|---|---|---|
| `&self` / `&mut self` / `self` | acesso do método | você **escolhe** ler/modificar/consumir |
| `fn` | ponteiro de função | tipo concreto, sem captura |
| `Fn`/`FnMut`/`FnOnce` | traits de chamável | closures que capturam ambiente |
| `&` / `&mut` / `*` | borrow / deref | sem GC; borrow checker força regras |
| `.clone()` | cópia explícita | nada é copiado implicitamente |
| `Vec<T>` / `&[T]` | array dono / slice | slice empresta sem copiar |
| `usize` | índice | obrigatório para indexar |
| `Option<T>` | talvez valor | substitui `null`, força tratamento |
| `Self` / `self` | tipo / instância | maiúsculo = tipo |
| sem `;` final | valor de retorno | `if`/`match` são expressões |
| `impl` | bloco de métodos | separado do `struct` |
| `pub` | visibilidade | privado por padrão |
| `#[derive(...)]` | gera trait | em compile-time |
| `nome!` | macro | não é função |
| `as` | cast | sem conversão numérica implícita |
| `mut` | mutável | imutável por padrão |
```
