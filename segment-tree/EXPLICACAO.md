# Segment Tree em Rust — Guia Linha a Linha

> Documento de estudo para reimplementar `src/loan_contracts.rs` do zero, entendendo **o algoritmo** e **cada palavra da linguagem Rust** usada.

---

## Índice

1. [O Algoritmo Segment Tree](#1-o-algoritmo-segment-tree)
2. [Vocabulário Rust que você precisa](#2-vocabulário-rust-que-você-precisa)
3. [O código, linha a linha](#3-o-código-linha-a-linha)
4. [O case: Loans (carteira de empréstimos)](#4-o-case-loans-carteira-de-empréstimos)
5. [Como rodar e testar](#5-como-rodar-e-testar)
6. [Referências](#6-referências)

---

## 1. O Algoritmo Segment Tree

### 1.1 Que problema resolve

Você tem um array de `n` elementos e quer responder, **rápido**, perguntas sobre **intervalos** (ranges):

- Qual a soma de `arr[l..=r]`?
- Qual o mínimo / máximo de `arr[l..=r]`?
- Qual o contrato mais urgente entre as posições `l` e `r`?

E você também quer **atualizar** um elemento (`arr[pos] = valor`) sem ter que recalcular tudo.

**Abordagem ingênua:** percorrer o intervalo a cada pergunta → `O(n)` por consulta. Se houver muitas consultas, fica caro.

**Segment Tree:** consulta e atualização em **`O(log n)`**. Construção em `O(n)`.

| Operação | Array ingênuo | Segment Tree |
|----------|---------------|--------------|
| Construir | `O(n)` | `O(n)` |
| Consultar intervalo | `O(n)` | `O(log n)` |
| Atualizar 1 posição | `O(1)` | `O(log n)` |

#### Entendendo a notação `O(...)`

Big-O mede **como o custo cresce conforme `n` cresce** (`n` = número de elementos), ignorando constantes.

| Notação | Nome | Passos para `n = 1.000.000` | Como cresce |
|---------|------|-----------------------------|-------------|
| `O(1)` | constante | 1 | nunca cresce |
| `O(log n)` | logarítmico | ~20 | devagar (dobrar `n` → +1 passo) |
| `O(n)` | linear | 1.000.000 | proporcional a `n` |

`log₂(1.000.000) ≈ 20`. É daí que vem a escalabilidade da Segment Tree.

#### As características de cada operação

- **`O(1)` — update do array ingênuo:** uma posição = uma escrita na memória (`arr[pos] = v`). Imbatível, mas não pré-computa nada agregado → a consulta paga `O(n)` depois.
- **`O(log n)` — query/update da Segment Tree:** toca apenas os nós no caminho folha↔raiz, ou seja, a **altura** da árvore (`log n`). Mais caro que `O(1)`, mas mantém os agregados prontos.
- **`O(n)` — query do array ingênuo:** percorre o intervalo inteiro. Intervalo de tamanho `k` → `k` passos; pior caso (intervalo todo) → `n` passos. Com `Q` consultas vira `O(Q·n)` e não escala.

#### O tradeoff central

A Segment Tree **piora** o update (`O(1)` → `O(log n)`) para **melhorar muito** a consulta (`O(n)` → `O(log n)`). Não é almoço grátis — é uma troca:

```
Array ingênuo:   update O(1)     barato  | query O(n)     caro
Segment Tree:    update O(log n) médio   | query O(log n) médio
```

> **"Barato" e "caro" aqui referem-se a tempo (CPU)** — quantos passos a CPU executa para completar a operação. `O(n)` = muitos passos = lento/caro; `O(1)` = um passo = rápido/barato. Não é sobre memória. Tempo e memória são **eixos separados**:
>
> | Eixo | Pergunta | Onde aparece neste doc |
> |------|----------|------------------------|
> | **Tempo** (CPU) | quantos passos para rodar? | tabela `O(1)/O(log n)/O(n)` — barato/caro |
> | **Espaço** (memória) | quanta RAM ocupa? | os `4*n` da árvore (§3.2) |
>
> A Segment Tree faz o clássico tradeoff *space-time*: **gasta memória (`4*n`) para comprar velocidade (`O(log n)` na consulta)**.

Custo escondido da árvore: `4*n` de memória extra (ver §3.2) + build `O(n)` na inicialização.

#### Regra de decisão — qual usar

| Perfil de carga | Melhor escolha | Por quê |
|-----------------|----------------|---------|
| Muitas consultas de intervalo **+** updates frequentes | **Segment Tree** | ambos `O(log n)`; nenhuma operação vira `O(n)` |
| Quase só update, raras consultas | **Array ingênuo** | update `O(1)` grátis |
| Array que **nunca** muda, só consulta soma | **Prefix sums** | query `O(1)`; mas update seria `O(n)` |

Resumo: a escolha depende do **perfil de leitura vs. escrita**. Quando há mistura equilibrada de consultas e atualizações, a árvore vence porque mantém **tudo** em `O(log n)`, sem deixar nenhuma operação degradar para `O(n)`.

### 1.2 A ideia central

A árvore divide o array em **segmentos** (intervalos). Cada **nó** representa um intervalo `[start, end]`:

- **Folhas** = elementos individuais (`[i, i]`).
- **Nós internos** = guardam o resultado de `merge(filho_esquerdo, filho_direito)`.
- A **raiz** representa o array inteiro `[0, n-1]`.

Exemplo com 8 elementos (índices 0..7). Cada nó mostra o intervalo que cobre:

```
                      [0..7]                <- raiz: agregado de tudo
              /                  \
         [0..3]                  [4..7]
        /      \                /      \
    [0..1]    [2..3]        [4..5]    [6..7]
    /   \     /   \         /   \     /   \
 [0] [1]  [2] [3]      [4] [5]  [6] [7]    <- folhas: elementos
```

Cada nível divide o intervalo ao meio → altura `log₂(n)` → daí o `O(log n)`.

### 1.3 As três operações

**BUILD (construir):** recursão de cima pra baixo. Se o nó é uma folha (`start == end`), copia o elemento. Senão, constrói os dois filhos e faz `merge` deles.

**QUERY (consultar `[l, r]`):** em cada nó há 3 casos:
1. **Sem sobreposição** — o intervalo do nó está totalmente fora de `[l, r]` → devolve o **elemento neutro** (não atrapalha o merge).
2. **Cobertura total** — o intervalo do nó está totalmente dentro de `[l, r]` → devolve o valor já agregado do nó (não precisa descer mais).
3. **Sobreposição parcial** — desce nos dois filhos e faz `merge` dos resultados.

**UPDATE (atualizar posição `pos`):** desce até a folha `pos`, troca o valor, e na volta recombina (`merge`) os nós no caminho de volta até a raiz. Só `O(log n)` nós mudam.

### 1.4 Requisito matemático: `merge` associativo + `neutral`

`merge` precisa ser **associativo**: `merge(a, merge(b, c)) == merge(merge(a, b), c)`. Isso garante que tanto faz a ordem em que combinamos os pedaços do intervalo.

`neutral` é o **elemento identidade**: `merge(x, neutral) == x`. Ele é devolvido para segmentos fora da consulta, sem alterar o resultado.

Em álgebra isso é um **monoide** (conjunto + operação associativa + identidade). Toda operação que forma monoide funciona em Segment Tree:

| Operação | `merge` | `neutral` |
|----------|---------|-----------|
| Soma | `a + b` | `0` |
| Mínimo | `min(a, b)` | `+∞` |
| Máximo | `max(a, b)` | `-∞` |
| Mais urgente (menor prazo) | contrato de menor `days_remaining` | contrato sentinela com `i32::MAX` |

---

## 2. Vocabulário Rust que você precisa

Aqui está **cada palavra/símbolo** da linguagem que aparece no código, explicada de forma isolada antes de ver as linhas.

### 2.1 `struct` — estrutura de dados

```rust
struct LoanContract {
    amount: f64,
}
```

`struct` define um tipo composto: um agrupamento de campos nomeados. Equivale a uma "classe sem métodos" (em Rust os métodos vêm separados num bloco `impl`). [Docs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html)

### 2.2 `Vec<T>` — vetor dinâmico

`Vec<T>` é um array que cresce em tempo de execução, alocado no heap. O `<T>` é o tipo dos elementos (`Vec<f64>` = vetor de floats). É o equivalente Rust de `ArrayList` (Java) ou `std::vector` (C++). [Docs](https://doc.rust-lang.org/std/vec/struct.Vec.html)

### 2.3 `<T>` — generics (tipo genérico)

`<T>` é um **parâmetro de tipo**. Em vez de escrever a árvore só para `f64`, escrevemos para um `T` qualquer; quem usa decide o tipo concreto. `T` é só um nome convencional (poderia ser `Item`). [Docs](https://doc.rust-lang.org/book/ch10-01-syntax.html)

### 2.4 `T: Clone` — trait bound (restrição de trait)

```rust
struct SegmentTree<T: Clone>
```

Lê-se: "para qualquer tipo `T` **que implemente o trait `Clone`**". Um **trait** é como uma interface: define capacidades. `Clone` significa "sei me copiar profundamente com `.clone()`". A restrição `T: Clone` é necessária porque a árvore copia valores ao combinar nós. [Traits](https://doc.rust-lang.org/book/ch10-02-traits.html) · [Clone](https://doc.rust-lang.org/std/clone/trait.Clone.html)

### 2.5 `impl` — bloco de implementação

```rust
impl<T: Clone> SegmentTree<T> { ... }
```

`impl` ("implement") é onde você escreve os **métodos** associados ao tipo. `impl<T: Clone> SegmentTree<T>` = "métodos para `SegmentTree` de qualquer `T` clonável". [Docs](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)

### 2.6 `fn` — função

`fn` declara uma função ou método. [Docs](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html)

### 2.7 `self`, `&self`, `&mut self` — o receptor do método

- `&self` — método **lê** a struct (empréstimo imutável). Não pode modificar.
- `&mut self` — método **modifica** a struct (empréstimo mutável).
- `self` (sem `&`) — método **consome** (toma posse) da struct.
- `Self` (com S maiúsculo) — o **tipo** atual (`SegmentTree<T>`), usado em retornos.

[Docs](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)

### 2.8 `&T` e `&` — referências (borrowing)

`&` cria uma **referência**: empresta um valor sem tomar posse dele. `&T` é "referência a um `T`". Isto está no coração do **ownership** de Rust: passar `&valor` evita cópia e mantém o dono original. [Docs](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)

### 2.9 `mut` — mutabilidade

Em Rust tudo é **imutável por padrão**. `let mut x` permite reatribuir/modificar `x`. Sem `mut`, o compilador recusa qualquer alteração. [Docs](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)

### 2.10 `let` — vincular variável

`let nome = valor;` cria uma vinculação (binding). Com `let mut`, vira mutável. [Docs](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)

### 2.11 Tipos numéricos: `usize`, `u32`, `i32`, `f64`

- `usize` — inteiro **sem sinal** do tamanho de um ponteiro (64 bits em máquina 64-bit). É o tipo usado para **índices** e **tamanhos** de coleções.
- `u32` — inteiro sem sinal de 32 bits (0 a ~4 bilhões).
- `i32` — inteiro **com sinal** de 32 bits (pode ser negativo).
- `f64` — ponto flutuante de 64 bits (double).

[Docs](https://doc.rust-lang.org/book/ch03-02-data-types.html)

### 2.12 `fn(&T, &T) -> T` — ponteiro de função

Este é um **tipo**: "uma função que recebe duas referências `&T` e devolve um `T`". `->` indica o tipo de retorno. Armazenar uma função como dado permite que a árvore receba a operação de merge de fora (estratégia plugável). [Docs](https://doc.rust-lang.org/book/ch19-05-advanced-functions-and-closures.html)

### 2.13 `String` vs `&str`

- `&str` — fatia de string emprestada, tamanho fixo, geralmente um literal (`"Alice"`).
- `String` — string **dona**, alocada no heap, pode crescer.
- `.to_string()` converte `&str` → `String`.

[Docs](https://doc.rust-lang.org/book/ch08-02-strings.html)

### 2.14 `vec![...]` — macro de criação de Vec

O `!` indica uma **macro** (não função). `vec![x; n]` cria um Vec com `n` cópias de `x`. `vec![a, b, c]` cria um Vec com esses elementos. [Docs](https://doc.rust-lang.org/std/macro.vec.html)

### 2.15 `#[derive(Clone)]` e `#![allow(dead_code)]` — atributos

`#[...]` é um **atributo** (metadado para o compilador).
- `#[derive(Clone)]` — gera **automaticamente** a implementação de `Clone` para a struct.
- `#![allow(dead_code)]` (com `!`) aplica ao **módulo inteiro**: silencia avisos de código não usado.

[Derive](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html) · [Attributes](https://doc.rust-lang.org/reference/attributes.html)

### 2.16 `return` e a expressão final sem `;`

Rust é **orientado a expressões**: a **última expressão sem ponto-e-vírgula** de um bloco é o valor de retorno. `return x;` é o retorno explícito (geralmente usado para sair cedo). [Docs](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#functions-with-return-values)

### 2.17 `pub` — visibilidade pública

`pub` torna o item visível fora do módulo. Sem `pub`, é privado ao módulo. [Docs](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html)

### 2.18 `println!` e `{}` — impressão formatada

`println!` é macro de impressão. `{}` é um placeholder preenchido pelos argumentos na ordem. `5_000.00` usa `_` só como separador visual de milhar (ignorado pelo compilador). [Docs](https://doc.rust-lang.org/std/macro.println.html)

### 2.19 `#[cfg(test)]`, `mod`, `#[test]`, `use super::*` — testes

- `mod tests { ... }` — declara um **módulo** chamado `tests`.
- `#[cfg(test)]` — compila esse módulo **só** durante `cargo test`.
- `#[test]` — marca uma função como teste.
- `use super::*;` — importa tudo (`*`) do módulo **pai** (`super`), para os testes enxergarem `SegmentTree` etc.
- `assert_eq!(a, b)` — falha o teste se `a != b`. `assert!(cond)` falha se `cond` for falso.

[Docs](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)

### 2.20 `i32::MAX`, `f64::INFINITY` — constantes associadas

`Tipo::CONSTANTE` acessa constantes do tipo. `i32::MAX` é o maior `i32`; `f64::INFINITY` é o infinito de ponto flutuante; `i32::MIN` e `f64::NEG_INFINITY` são os extremos opostos. Usados como **sentinelas** para o `neutral`. [i32](https://doc.rust-lang.org/std/primitive.i32.html) · [f64](https://doc.rust-lang.org/std/primitive.f64.html)

### 2.21 Iteradores: `.iter()`, `.map()`, `.collect()`, `|c| ...`

```rust
init_contracts().iter().map(|c| c.amount).collect()
```

- `.iter()` — cria um iterador de referências aos elementos.
- `.map(|c| c.amount)` — transforma cada elemento; `|c| c.amount` é uma **closure** (função anônima) que recebe `c` e devolve `c.amount`.
- `.collect()` — junta o resultado do iterador numa coleção (aqui, um `Vec<f64>`).

[Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html) · [Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)

---

## 3. O código, linha a linha

### 3.1 A struct `SegmentTree`

```rust
struct SegmentTree<T: Clone> {
    tree:    Vec<T>,
    n:       usize,
    neutral: T,
    merge:   fn(&T, &T) -> T,
}
```

- `struct SegmentTree<T: Clone>` — define a árvore genérica sobre `T`, exigindo que `T` seja `Clone` (a árvore copia valores ao combinar nós).
- `tree: Vec<T>` — o array que **armazena a árvore inteira**. Em vez de ponteiros/nós, usamos um Vec indexado: o filho esquerdo de `node` fica em `2*node`, o direito em `2*node+1`. Memória contígua = mais rápido em cache.
- `n: usize` — quantos elementos de dado existem (folhas reais).
- `neutral: T` — o elemento identidade do merge.
- `merge: fn(&T, &T) -> T` — a função que combina dois valores. Guardada como dado, então a mesma struct serve para soma, min, max etc.

### 3.2 `new` — construtor

```rust
impl<T: Clone> SegmentTree<T> {
    fn new(data: Vec<T>, neutral: T, merge: fn(&T, &T) -> T) -> Self {
        let n = data.len();
        let tree = vec![neutral.clone(); 4 * n];
        let mut st = SegmentTree { tree, n, neutral, merge };
        if n > 0 {
            st.build(&data, 1, 0, n - 1);
        }
        st
    }
```

- `fn new(...) -> Self` — função associada (sem `self`) que devolve um `Self` (= `SegmentTree<T>`). Convenção Rust para construtores.
- `data: Vec<T>` — recebe **posse** do vetor de dados.
- `let n = data.len()` — `.len()` devolve o tamanho do vetor como `usize`.
- `let tree = vec![neutral.clone(); 4 * n]` — pré-aloca `4*n` posições, todas com cópias do neutro. **Por que `4*n`?** Garante espaço suficiente para a indexação `2*node`/`2*node+1` mesmo quando `n` não é potência de 2 (`2*n` poderia estourar; `4*n` é o limite seguro clássico). `.clone()` é preciso porque cada slot precisa do próprio valor.
- `let mut st = SegmentTree { tree, n, neutral, merge }` — cria a instância. **Field init shorthand**: como as variáveis se chamam igual aos campos (`tree`, `n`...), não precisa escrever `tree: tree`. Precisa de `mut` porque `build` vai modificar `st`.
- `if n > 0` — protege contra árvore vazia (com `n == 0`, `n - 1` causaria underflow em `usize`, que não tem negativos).
- `st.build(&data, 1, 0, n - 1)` — inicia a construção na **raiz** (`node = 1`), cobrindo o intervalo `[0, n-1]`. Passa `&data` (referência, não move). Começamos em `node = 1` (não 0) para a aritmética `2*node`/`2*node+1` funcionar.
- `st` — última expressão sem `;`: retorna a árvore construída.

### 3.3 `query` — fachada de consulta

```rust
    fn query(&self, l: usize, r: usize) -> T {
        self.query_range(1, 0, self.n - 1, l, r)
    }
```

- `&self` — só lê a árvore.
- Delega para `query_range`, começando na raiz (`node=1`) cobrindo `[0, n-1]`, buscando o intervalo do usuário `[l, r]`. Separar a API pública (`query`) da recursão (`query_range`) deixa a interface limpa.

### 3.4 `update` — fachada de atualização

```rust
    fn update(&mut self, pos: usize, value: T) {
        let n = self.n;
        self.update_range(1, 0, n - 1, pos, value);
    }
```

- `&mut self` — vai modificar a árvore.
- `let n = self.n` — copia `self.n` para uma variável local. Isso evita um conflito de **borrow**: chamar `self.update_range(..., self.n - 1, ...)` enquanto já se empresta `self` mutavelmente causaria erro do borrow checker; com `n` local, lemos antes.
- Inicia a recursão de update na raiz.

### 3.5 `root` e `len` — acessores

```rust
    fn root(&self) -> T {
        self.tree[1].clone()
    }

    fn len(&self) -> usize {
        self.n
    }
```

- `root` — devolve o valor agregado de **toda** a árvore (índice `1` é a raiz). `.clone()` porque devolvemos uma cópia, não a referência interna.
- `len` — devolve quantos elementos a árvore guarda.

### 3.6 `build` — construção recursiva

```rust
    fn build(&mut self, data: &[T], node: usize, start: usize, end: usize) {
        if start == end {
            self.tree[node] = data[start].clone();
            return;
        }
        let mid = (start + end) / 2;
        self.build(data, 2 * node,     start,   mid);
        self.build(data, 2 * node + 1, mid + 1, end);
        let left  = self.tree[2 * node].clone();
        let right = self.tree[2 * node + 1].clone();
        self.tree[node] = (self.merge)(&left, &right);
    }
```

- `data: &[T]` — `&[T]` é uma **slice** (fatia emprestada do vetor); mais geral que `&Vec<T>`.
- `if start == end` — **caso base**: intervalo de 1 elemento = folha. Copia o dado para o nó. `return;` sai cedo.
- `let mid = (start + end) / 2` — ponto médio (divisão inteira) para dividir o intervalo.
- `self.build(data, 2 * node, start, mid)` — constrói o **filho esquerdo** (índice `2*node`), cobrindo `[start, mid]`.
- `self.build(data, 2 * node + 1, mid + 1, end)` — constrói o **filho direito** (índice `2*node+1`), cobrindo `[mid+1, end]`.
- `let left = self.tree[2 * node].clone()` / `right` — pega os valores já calculados dos filhos. Clonamos para soltar o empréstimo de `self.tree` antes de escrever nele de novo.
- `self.tree[node] = (self.merge)(&left, &right)` — combina os filhos e guarda no nó pai. `(self.merge)` precisa de parênteses para o compilador entender que é o **campo função** sendo chamado (e não um método chamado `merge`).

### 3.7 `query_range` — consulta recursiva (o coração)

```rust
    fn query_range(&self, node: usize, start: usize, end: usize, l: usize, r: usize) -> T {
        if r < start || end < l { return self.neutral.clone(); }
        if l <= start && end <= r { return self.tree[node].clone(); }
        let mid   = (start + end) / 2;
        let left  = self.query_range(2 * node,     start,   mid, l, r);
        let right = self.query_range(2 * node + 1, mid + 1, end, l, r);
        (self.merge)(&left, &right)
    }
```

Os parâmetros: `node` é o nó atual; `[start, end]` é o que esse nó cobre; `[l, r]` é o que o usuário quer.

- `if r < start || end < l` — **caso 1: sem sobreposição**. Se o intervalo pedido termina antes do nó começar (`r < start`) ou começa depois do nó terminar (`end < l`), o nó não contribui → devolve `neutral` (identidade, não atrapalha). `||` é OU lógico.
- `if l <= start && end <= r` — **caso 2: cobertura total**. O nó está inteiramente dentro de `[l, r]` → devolve o valor já agregado, sem descer. `&&` é E lógico. **Esta é a chave do `O(log n)`**: cortamos subárvores inteiras.
- senão, **caso 3: sobreposição parcial** — calcula `mid`, consulta os dois filhos recursivamente, e faz `merge` dos resultados (última expressão, sem `;`).

### 3.8 `update_range` — atualização recursiva

```rust
    fn update_range(&mut self, node: usize, start: usize, end: usize, pos: usize, value: T) {
        if start == end {
            self.tree[node] = value;
            return;
        }
        let mid = (start + end) / 2;
        if pos <= mid {
            self.update_range(2 * node,     start,   mid, pos, value);
        } else {
            self.update_range(2 * node + 1, mid + 1, end, pos, value);
        }
        let left  = self.tree[2 * node].clone();
        let right = self.tree[2 * node + 1].clone();
        self.tree[node] = (self.merge)(&left, &right);
    }
```

- `if start == end` — chegou na folha alvo: escreve o novo valor. (Aqui `value` é movido para dentro do nó, por isso não precisa de `.clone()`.)
- `if pos <= mid { ... } else { ... }` — decide **para qual lado descer**: se a posição está na metade esquerda, desce à esquerda; senão à direita. Só **um** caminho é percorrido → `O(log n)`.
- depois de atualizar a folha, na **volta da recursão**, recombina os filhos e regrava o pai. Assim só os nós no caminho folha→raiz são recalculados.

### 3.9 `LoanContract` — o dado do domínio

```rust
#[derive(Clone)]
struct LoanContract {
    contract_id:    u32,
    borrower:       String,
    amount:         f64,
    days_remaining: i32,
}
```

- `#[derive(Clone)]` — gera `Clone` automaticamente (a árvore exige `T: Clone`).
- `contract_id: u32` — id do contrato (inteiro positivo).
- `borrower: String` — nome do tomador (string dona).
- `amount: f64` — valor do empréstimo.
- `days_remaining: i32` — dias até o vencimento (com sinal; `i32` permite as sentinelas `MAX`/`MIN`).

```rust
impl LoanContract {
    fn new(contract_id: u32, borrower: &str, amount: f64, days_remaining: i32) -> Self {
        LoanContract {
            contract_id,
            borrower: borrower.to_string(),
            amount,
            days_remaining,
        }
    }
}
```

- Construtor. Recebe `borrower: &str` (literal emprestado) e converte com `.to_string()` para `String` dona. Os outros campos usam o shorthand.

### 3.10 Dados iniciais

```rust
fn init_contracts() -> Vec<LoanContract> {
    vec![
        LoanContract::new(1, "Alice",  5_000.00, 30),
        LoanContract::new(2, "Bob",   12_000.00,  7),
        ...
    ]
}
```

- `Tipo::funcao()` chama uma função **associada** (sintaxe `::`). Monta a carteira de 8 contratos com `vec![...]`.

### 3.11 As estratégias de merge (o que torna a árvore reutilizável)

```rust
fn neutral_urgent() -> LoanContract { LoanContract::new(0, "-", 0.0, i32::MAX) }
fn merge_urgent(a: &LoanContract, b: &LoanContract) -> LoanContract {
    if a.days_remaining <= b.days_remaining { a.clone() } else { b.clone() }
}
```

- `merge_urgent` — "mais urgente" = **menor** `days_remaining`. Compara os dois e devolve a cópia do menor. Como `if/else` é uma **expressão** em Rust, o bloco `{ a.clone() }` ou `{ b.clone() }` vira o valor retornado (sem `return`, sem `;`).
- `neutral_urgent` — sentinela com `days_remaining = i32::MAX`: como nada é "mais urgente que infinito de dias", ele nunca vence a comparação → é a identidade do mínimo.

Análogos:
- `merge_slack` / `neutral_slack` — "mais folga" = **maior** prazo; sentinela `i32::MIN`.
- `merge_lowest` / `neutral_lowest` — menor `amount`; sentinela `f64::INFINITY`.
- `merge_highest` / `neutral_highest` — maior `amount`; sentinela `f64::NEG_INFINITY`.
- `merge_f64_sum(a, b) = a + b` — soma simples sobre `f64`; neutro `0.0`.

Repare: **a mesma `SegmentTree` faz min, max, soma e seleção de objeto** só trocando o par `(neutral, merge)`. É o ponto alto da apresentação.

### 3.12 `run` — demonstração

```rust
pub fn run() {
    println!("Loan contracts module running");

    let urgent_tree = SegmentTree::new(
        init_contracts(),
        neutral_urgent(),
        merge_urgent,
    );

    let most_urgent = urgent_tree.root();

    println!(
        "Most urgent contract => id={}, borrower={}, days_remaining={}",
        most_urgent.contract_id,
        most_urgent.borrower,
        most_urgent.days_remaining
    );
}
```

- `pub fn run()` — público (chamado pelo `main`).
- Constrói a árvore "urgente" e pega a `root()` = o contrato mais urgente de toda a carteira (Eve, 2 dias).
- `merge_urgent` é passado **sem parênteses** — é o **ponteiro da função**, não a chamada dela.

### 3.13 Os testes

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_urgent_tree() -> SegmentTree<LoanContract> {
        SegmentTree::new(init_contracts(), neutral_urgent(), merge_urgent)
    }

    #[test]
    fn urgent_root_is_eve() {
        let st = make_urgent_tree();
        assert_eq!(st.root().contract_id, 5);
        assert_eq!(st.root().days_remaining, 2);
    }
    ...
```

- `#[cfg(test)] mod tests` — módulo compilado só em `cargo test`.
- `use super::*` — traz `SegmentTree`, `LoanContract`, os merges etc. do módulo pai.
- `make_urgent_tree()` — helper que monta a árvore de teste.
- `assert_eq!(st.root().contract_id, 5)` — afirma que o mais urgente é a Eve (id 5, 2 dias). Se não for, o teste falha mostrando os dois valores.
- Outro teste interessante, `urgent_after_eve_renegotiation_bob_takes_over`: faz `st.update(4, ...)` renegociando a Eve para 90 dias e verifica que o novo mais urgente vira o Bob (id 2, 7 dias) — demonstra **update + query** juntos.

---

## 4. O case: Loans (carteira de empréstimos)

Os 8 contratos:

| idx | id | tomador | amount | days_remaining |
|-----|----|---------|--------|----------------|
| 0 | 1 | Alice |  5 000 | 30 |
| 1 | 2 | Bob   | 12 000 |  7 |
| 2 | 3 | Carol |  3 200 | 45 |
| 3 | 4 | David |  8 500 | 14 |
| 4 | 5 | Eve   |  2 100 |  2 |
| 5 | 6 | Frank |  9 900 | 21 |
| 6 | 7 | Grace |  6 700 | 60 |
| 7 | 8 | Hank  |  4 400 |  9 |

Perguntas que a Segment Tree responde em `O(log n)`:

- **Contrato mais urgente** (menor prazo) na carteira ou num intervalo → `merge_urgent`. Resposta global: Eve (2 dias).
- **Mais folga** (maior prazo) → `merge_slack`. Global: Grace (60 dias).
- **Menor / maior valor** → `merge_lowest` / `merge_highest`.
- **Soma da carteira** (ou de um trecho) → `merge_f64_sum`. Total: 51 800.
- **Renegociação** (mudar um contrato) → `update`, recalculando só `O(log n)` nós.

Por que Segment Tree e não um simples `for`? Numa carteira com milhões de contratos e muitas consultas por intervalo + atualizações frequentes, o `for` ingênuo (`O(n)` por consulta) não escala; a árvore mantém tudo em `O(log n)`. (Para o tamanho do kata, é didático — veja os **Cons** no README: sem validação de limites, `4*n` de memória, `f64` para dinheiro não é ideal em produção, etc.)

---

## 5. Como rodar e testar

```bash
cargo run     # executa run() -> imprime o contrato mais urgente
cargo test    # roda todos os testes
```

Saída esperada do `run`:

```text
Loan contracts module running
Most urgent contract => id=5, borrower=Eve, days_remaining=2
```

---

## 6. Referências

### Algoritmo Segment Tree
- **CP-Algorithms — Segment Tree** (a referência canônica, com build/query/update e lazy propagation): https://cp-algorithms.com/data_structures/segment_tree.html
- **USACO Guide — Point Update Range Sum**: https://usaco.guide/gold/PURS
- **GeeksforGeeks — Segment Tree (intro + somas)**: https://www.geeksforgeeks.org/segment-tree-data-structure/
- **Vídeo — Segment Tree (Tushar Roy / William Fiset)**: https://www.youtube.com/watch?v=ZBHKZF5w4YU

### Rust — linguagem
- **The Rust Book** (gratuito, oficial — leia caps. 3, 4, 5, 10, 11, 13): https://doc.rust-lang.org/book/
- **Rust by Example** (exemplos executáveis): https://doc.rust-lang.org/rust-by-example/
- **Rustlings** (exercícios práticos no terminal): https://github.com/rust-lang/rustlings

### Tópicos Rust específicos deste código
| Conceito | Link |
|----------|------|
| Structs | https://doc.rust-lang.org/book/ch05-01-defining-structs.html |
| `impl` e métodos | https://doc.rust-lang.org/book/ch05-03-method-syntax.html |
| Ownership | https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html |
| Referências e borrowing | https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html |
| Generics `<T>` | https://doc.rust-lang.org/book/ch10-01-syntax.html |
| Traits e trait bounds (`T: Clone`) | https://doc.rust-lang.org/book/ch10-02-traits.html |
| `Vec<T>` | https://doc.rust-lang.org/std/vec/struct.Vec.html |
| Funções e closures como valores | https://doc.rust-lang.org/book/ch19-05-advanced-functions-and-closures.html |
| Iteradores (`map`/`collect`) | https://doc.rust-lang.org/book/ch13-02-iterators.html |
| Testes | https://doc.rust-lang.org/book/ch11-01-writing-tests.html |
| Macros (`vec!`, `println!`) | https://doc.rust-lang.org/book/ch19-06-macros.html |

### Curso
- **Comprehensive Rust (Google, gratuito, com slides — ótimo para apresentação)**: https://google.github.io/comprehensive-rust/
