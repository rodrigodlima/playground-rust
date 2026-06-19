# A Carteira de Empréstimos — a história da solução

> Versão **simples e narrativa** para apresentar ao time, **na ordem em que o código aparece** em [`src/loan_contracts.rs`](./src/loan_contracts.rs) — feita para acompanhar rolando o arquivo de cima pra baixo na tela. Para o aprofundamento (vocabulário Rust, matemática, complexidade), veja [`EXPLICACAO.md`](./EXPLICACAO.md).

---

## A história (abertura — antes de mostrar o código)

Imagine que você gerencia uma **carteira de empréstimos** no banco. São 8 contratos hoje, mas poderiam ser milhões. Cada dia o chefe chega com perguntas:

- *"Qual cliente está mais perto de vencer?"*
- *"Entre os contratos 1 e 4, qual é o maior valor?"*
- *"Quanto soma a carteira inteira?"*
- *"A Eve renegociou — atualiza aí e me diz quem virou o mais urgente."*

Responder cada pergunta percorrendo a lista toda funciona com 8 contratos. **Não funciona com milhões.** Entra a **Segment Tree**: estrutura que responde essas perguntas (e aceita atualizações) de forma rápida, porque guarda **resumos pré-calculados** de pedaços da carteira.

Agora vamos descer pelo arquivo e ver isso virar código.

---

## 1. O dado do domínio — `struct LoanContract` (topo do arquivo)

A primeira coisa no arquivo é o que tudo gira em torno: o contrato. Quatro campos:

```rust
#[derive(Clone)]
struct LoanContract {
    contract_id:    u32,     // id do contrato
    borrower:       String,  // nome do cliente
    amount:         f64,     // valor emprestado
    days_remaining: i32,     // dias até vencer
}
```

Logo abaixo, um construtor (`impl LoanContract`) que monta um contrato a partir dos campos:

```rust
impl LoanContract {
    fn new(contract_id: u32, borrower: &str, amount: f64, days_remaining: i32) -> Self { ... }
}
```

> `#[derive(Clone)]` = "esse contrato sabe se copiar". A árvore vai exigir isso (ela copia valores ao combinar pedaços).

---

## 2. A estrutura da árvore — `struct SegmentTree` + `impl`

Depois do dado, vem a árvore. Por dentro, ela é **um único array** chamado `tree`:

```rust
struct SegmentTree<T: Clone> {
    tree:    Vec<T>,          // <- AQUI moram os dados + os resumos
    n:       usize,           // quantos contratos
    neutral: T,               // o "valor vazio" (explico na parte 4)
    merge:   fn(&T, &T) -> T, // a regra de combinar dois itens (parte 4)
}
```

A sacada: a árvore não guarda só os 8 contratos. Guarda também **resumos** de pedaços da carteira. Visualmente:

```
                      [tudo]                 <- resumo da carteira inteira
              /                  \
         [0..3]                  [4..7]      <- resumo de cada metade
        /      \                /      \
    [0..1]    [2..3]        [4..5]    [6..7]
    /   \     /   \         /   \     /   \
 Alice Bob Carol David  Eve Frank Grace Hank  <- os contratos de verdade
```

Cada caixa de cima é o resultado de **combinar** (`merge`) as duas de baixo. Tudo isso mora no array `tree`: o filho esquerdo de um nó fica em `2*node`, o direito em `2*node+1`.

> O `<T: Clone>` torna a árvore **genérica**: serve pra qualquer tipo (contrato, `f64`...), desde que saiba se clonar.

### As operações — o bloco `impl`

São 3 ações principais.

#### 🔨 Montar a carteira — `new` + `build`

Criar a árvore monta os resumos **uma vez**:

```rust
fn new(data: Vec<T>, neutral: T, merge: fn(&T, &T) -> T) -> Self {
    let n = data.len();
    let tree = vec![neutral.clone(); 4 * n]; // reserva espaço (4*n é o teto seguro)
    let mut st = SegmentTree { tree, n, neutral, merge };
    if n > 0 { st.build(&data, 1, 0, n - 1); } // preenche a árvore
    st
}
```

`build` preenche de baixo pra cima: cada contrato vira folha, e vai combinando até o topo:

```rust
fn build(&mut self, data, node, start, end) {
    if start == end {                          // chegou num contrato individual
        self.tree[node] = data[start].clone(); // guarda ele
        return;
    }
    let mid = (start + end) / 2;
    self.build(data, 2*node,   start, mid);     // monta metade esquerda
    self.build(data, 2*node+1, mid+1, end);     // monta metade direita
    self.tree[node] = (self.merge)(&left, &right); // resumo do pedaço
}
```

> Pague esse custo **uma vez** na montagem. Depois, toda pergunta é barata.

#### 🔍 Consultar — `query` + `query_range`

Quer a resposta de um intervalo? `query(início, fim)`:

```rust
fn query(&self, l: usize, r: usize) -> T {
    self.query_range(1, 0, self.n - 1, l, r)
}
```

A mágica do `query_range`: ele **não percorre tudo**. Se um pedaço já tem resumo pronto e cabe inteiro na pergunta, usa o resumo e **nem desce** naquele galho:

```rust
fn query_range(&self, node, start, end, l, r) -> T {
    if r < start || end < l { return self.neutral.clone(); }   // pedaço fora: ignora
    if l <= start && end <= r { return self.tree[node].clone(); } // pedaço todo dentro: usa resumo pronto!
    let left  = self.query_range(2*node,   start, mid, l, r);  // só em parte: desce
    let right = self.query_range(2*node+1, mid+1, end, l, r);
    (self.merge)(&left, &right)
}
```

> Cortar galhos inteiros = consulta rápida mesmo com milhões de contratos.

#### ✏️ Atualizar — `update` + `update_range`

A Eve renegociou. Troca o contrato dela (índice 4):

```rust
fn update(&mut self, pos: usize, value: T) {
    let n = self.n;
    self.update_range(1, 0, n - 1, pos, value);
}
```

`update_range` desce até a folha da Eve, troca, e **na volta só refaz os resumos do caminho até o topo** — não mexe no resto:

```rust
fn update_range(&mut self, node, start, end, pos, value) {
    if start == end { self.tree[node] = value; return; } // achou a folha: troca
    if pos <= mid { self.update_range(2*node, ...) }      // desce SÓ pro lado certo
    else          { self.update_range(2*node+1, ...) }
    self.tree[node] = (self.merge)(&left, &right);        // na volta: refaz o resumo
}
```

> Atualizar um contrato **não** recalcula a carteira inteira. Só o caminho dele até o topo.

#### Atalhos — `root` e `len`

```rust
fn root(&self) -> T { self.tree[1].clone() } // resumo do topo = resposta da carteira inteira
fn len(&self)  -> usize { self.n }            // quantos contratos
```

---

## 3. A carteira inicial — `init_contracts`

A lista dos 8 contratos de exemplo:

```rust
fn init_contracts() -> Vec<LoanContract> {
    vec![
        LoanContract::new(1, "Alice",  5_000.00, 30),
        LoanContract::new(2, "Bob",   12_000.00,  7),
        // ... 8 no total
    ]
}
```

| idx | id | cliente | valor | dias p/ vencer |
|-----|----|---------|-------|----------------|
| 0 | 1 | Alice |  5 000 | 30 |
| 1 | 2 | Bob   | 12 000 |  7 |
| 2 | 3 | Carol |  3 200 | 45 |
| 3 | 4 | David |  8 500 | 14 |
| 4 | 5 | Eve   |  2 100 |  2 |
| 5 | 6 | Frank |  9 900 | 21 |
| 6 | 7 | Grace |  6 700 | 60 |
| 7 | 8 | Hank  |  4 400 |  9 |

> 💡 As perguntas por intervalo usam o **índice** (idx), não o id. "Contratos 0 a 3" = Alice, Bob, Carol, David.

---

## 4. O truque que amarra tudo — as funções `merge` + `neutral`

Aqui se resolve o suspense das partes 1 e 2. A **mesma árvore** responde perguntas diferentes — só muda a regra de combinar (`merge`) e o valor vazio (`neutral`):

```rust
// "Mais urgente" = quem tem MENOS dias pra vencer
fn neutral_urgent() -> LoanContract { LoanContract::new(0, "-", 0.0, i32::MAX) }
fn merge_urgent(a, b) { if a.days_remaining <= b.days_remaining { a } else { b } }

// "Maior valor" = quem tem MAIS amount
fn merge_highest(a, b) { if a.amount >= b.amount { a } else { b } }

// "Soma" = somar os valores
fn merge_f64_sum(a, b) { a + b }
```

| Pergunta do chefe | regra (`merge`) | resposta global |
|-------------------|-----------------|-----------------|
| Mais urgente (menos dias) | `merge_urgent` | Eve (2 dias) |
| Mais folga (mais dias) | `merge_slack` | Grace (60 dias) |
| Menor valor | `merge_lowest` | Eve (2 100) |
| Maior valor | `merge_highest` | Bob (12 000) |
| Soma da carteira | `merge_f64_sum` | 51 800 |

> **Ponto alto:** escrevemos a estrutura **uma vez** e ela serve pra min, max, soma e seleção de contrato — só trocando duas linhas.

E o `neutral`? É o "valor vazio" devolvido para pedaços fora da pergunta, escolhido pra **não atrapalhar**. Pro "mais urgente", o vazio é um contrato com `i32::MAX` dias (infinitos dias → nunca é o mais urgente). Pra soma, o vazio é `0`.

---

## 5. Juntando tudo — `run` (a demo completa)

No fim do arquivo, `run` mostra **todas as perguntas** trabalhando juntas. Ele monta uma árvore por estratégia e imprime cada resposta:

```rust
pub fn run() {
    // 1. mais urgente (toda a carteira)
    let urgent_tree = SegmentTree::new(init_contracts(), neutral_urgent(), merge_urgent);
    println!("Most urgent => {}", urgent_tree.root().borrower); // Eve

    // 2. mais folga | 3. menor valor | 4. maior valor — mesma ideia, só trocando merge/neutral
    let slack_tree   = SegmentTree::new(init_contracts(), neutral_slack(),   merge_slack);
    let lowest_tree  = SegmentTree::new(init_contracts(), neutral_lowest(),  merge_lowest);
    let highest_tree = SegmentTree::new(init_contracts(), neutral_highest(), merge_highest);

    // 5. consulta por intervalo — mais urgente só na primeira metade [0..3]
    let first_half = urgent_tree.query(0, 3); // Bob

    // 6. update — Eve renegocia (2 -> 90 dias), o topo muda
    let mut t = SegmentTree::new(init_contracts(), neutral_urgent(), merge_urgent);
    t.update(4, LoanContract::new(5, "Eve", 1500.0, 90)); // agora o mais urgente é o Bob

    // 7. soma da carteira (árvore de f64), e soma após pagamento parcial
    let amounts: Vec<f64> = init_contracts().iter().map(|c| c.amount).collect();
    let mut sum_tree = SegmentTree::new(amounts, 0.0, merge_f64_sum);
    println!("Total = {}", sum_tree.root()); // 51 800
    sum_tree.update(4, 500.0);                // Eve paga, total cai
}
```

Repare: **as 7 demos usam a mesma `SegmentTree`** — só trocam o par `(neutral, merge)` ou chamam `query`/`update`. É a prova viva do truque da parte 4.

```bash
cargo run    # imprime as 7 seções: urgência, folga, menor, maior, query, update, soma
```

---

## 6. As provas — os testes (`mod tests`)

Por último, os testes provam cada pergunta da história. O mais legal:

```rust
#[test]
fn urgent_after_eve_renegotiation_bob_takes_over() {
    let mut st = make_urgent_tree();
    st.update(4, LoanContract::new(5, "Eve", 1_500.00, 90)); // Eve renegocia: 2 -> 90 dias
    let r = st.query(0, 7);                                   // pergunta de novo
    assert_eq!(r.contract_id, 2);                             // agora o mais urgente é o Bob
}
```

> **Update + query juntos numa história só:** a Eve era a mais urgente; depois de renegociar, o Bob assume.

```bash
cargo test   # roda todos os testes que provam cada pergunta
```

---

## Fechando

- **Contrato** (`LoanContract`) = o dado.
- **Árvore** (`SegmentTree`) = array que guarda os dados **e resumos prontos** de cada pedaço.
- **3 ações:** montar (`build`, uma vez), consultar (`query`, rápido), atualizar (`update`, rápido).
- **`merge` plugável** faz a mesma árvore responder mil perguntas (a demo `run` mostra 7).

**Por que não um `for` simples?** Com 8 contratos, tanto faz. Com milhões + muitas perguntas por dia + atualizações constantes, o `for` percorre tudo toda vez e não escala. A árvore mantém tudo rápido.

> Nível profundo (cada palavra de Rust, monoide, complexidade `O(log n)`)? → [`EXPLICACAO.md`](./EXPLICACAO.md).
