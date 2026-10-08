# How this tool explains itself

> **Answer first.** Everything in this tool that teaches a person something — a node's page,
> a result, the manual, these documents, the figures — follows one standard: the
> answer first, then the thing said simply, then the real thing with its source, then where the
> simple version breaks. Every claim says whether it is sourced, derived, declared or only an
> example, and every block says what kind of reading it is. The rules are below, each with where
> it applies and what checks it; a rule nothing checks is marked so.
>
> **Kind:** reference + explanation · **For:** everyone

The standard is taken from *Eight Loops, One Beam* — a teaching page that explains an RF ion
thruster and its eight-loop controller with the Feynman technique and thirteen techniques from
the learning-science literature layered on it. That page takes one example and runs every
technique on it; this tool takes the same techniques and runs them on every node, form, result
and page. Where a technique is about a *learner's* own work (writing an explanation before
reading one), it appears here as the *Learn* depth; where it is about how the *explainer* lays
a page out, it is a rule every surface follows.

---

## Why

A clear page someone else wrote feels like understanding and often is not: people rate their
understanding of how things work far higher before they try to explain it step by step than
after (the *illusion of explanatory depth*, Rozenblit & Keil 2002). A design tool is read under
exactly that illusion. A number with a unit and three decimals looks understood. So the tool
does not only show numbers: it says what each one is, in words, where it comes from, and where
it stops being true — and it holds its own surfaces to that, by check, the way it holds the
physics to its fixtures.

---

## The rules

Each rule has an id so a check, a review comment or a commit can name it.

### E1 · Answer first

Lead with the answer in one or two sentences, then at most four supporting points, then the
detail. Busy readers read top-down; a page that builds up to its answer is read by nobody who
needed it. *(Minto, the pyramid principle.)*

**Where:** the *Answer first* box on every node page · the first lines of a result, in the
Results view and the HTML report · the answer line at the top of every manual section · the
first block of every document under `docs/` and at the top of the repository · the live answer
under each figure's question.
**Checked by:** the page generator's test (`the_page_follows_the_standard`), the result test
(`a_result_keeps_the_beliefs_it_rests_on…`: the answer comes before any table), the manual's
parser (a section without `answer` is refused), `tools/docs_lint.py` (a document without an
*Answer first* block fails).

### E2 · The station order: say it simply → the real thing → where it breaks → try it

Every concept is met in the same four steps. **Say it simply**: plain words, no symbol, no
term a newcomer would have to look up. **Now the real thing**: the relation as the source states
it, with the source. **Where the simple version breaks**: the place the plain words stop being
true. **Try it**: a live model, a run, a sweep. *(The Feynman technique; Mayer's segmenting.)*

**Where:** the first tab of every node page (*said simply, then the real thing*) · the node
application (`web/node.html`), whose explanation step asks for the plain words, and where they
break, before its theory step asks for the relation · a result's report (*Said simply*, *Where it breaks*, then the tables).
**Checked by:** the page generator's test (the four headings, in order); the gap pass lists
every published row that is not yet said simply, or not where that breaks, as an open gap —
so it holds the row back from review until it is written.

### E3 · Every claim says what kind of claim it is

Four kinds, marked on the claim itself:

| mark | means |
|---|---|
| **sourced** · *id* | stated in the cited work — `sources/` has the entry |
| **derived** | worked here from sourced relations |
| **declared** · *who* | a number a person chose, with their name against it |
| **illustrative** | an example or a widget value, not data |

**Where:** the relation on a node page (sourced), each theory step (derived), a declared value
(declared), the result report's *Said simply*. **Checked by:** the page generator's test; the gate already refuses a relation whose
source does not resolve (V8) and a declared value with nobody's name.

### E4 · Every simplification says where it stops being true

The plain words, an analogy, a picture: each is wrong somewhere, and saying where is what makes
it safe to use. *(Eight Loops' "where the story lies"; structure mapping, Gentner 1983.)*

**Where:** *Where the simple version breaks* on every node; every assumption's `fails_when`;
every figure's *Where the picture stops being true* line; a result's *Where it breaks*.
**Checked by:** the schema refuses an assumption without `fails_when`; the gap pass holds a
row without `[explain] breaks`; `docs_lint` fails a figure in `web/js/solar.js` without a
`breaks` line.

### E5 · Name the common wrong idea, then correct it

State the misconception, say why it is wrong, give the right idea. It targets the gaps most
readers share rather than leaving each to rediscover them. *(Refutation text, Tippett 2010.)*

**Where:** *Common wrong idea* on a node page, from `[explain] wrong`. Optional: not every row
has one. **Checked by:** nothing — the node application asks for it (*the common misreading*) and the
page shows it where written.

### E6 · Overview first, zoom, details on demand

The whole first, then a way in, then the detail only when asked. *(Shneiderman 1996.)*

**Where:** the tree and matrix before a node; the *Answer first* box before a node's tabs; a
result's answer before its tables; the manual's layers before its sections.
**Checked by:** the same tests as E1.

### E7 · Parts before process

Name the parts before the process that joins them. *(Mayer's pre-training principle.)*

**Where:** a node page's *assembly* and *connectivity* segments come before its sheet; the node
application shows what the node answers, what feeds it and who reads it before it asks the
relation; the interface tab lists every input before the algorithm tab composes them.
**Checked by:** the order is fixed in `web/js/node.js` and `web/js/napp.js`; nothing else checks
it.

### E8 · One kind of documentation per block, and it says which

*Tutorial* (learn by doing), *how-to* (a recipe for a goal), *reference* (facts to look up),
*explanation* (why). Mixed in one block, each gets in the way of the others. *(Diátaxis.)*

**Where:** every node page tab carries its kind; every manual section declares `kind`; every
document declares its **Kind** in its first block; the result report's sections are marked.
**Checked by:** the page generator's test; the manual's parser refuses a section without a
kind or with any other; `docs_lint` fails a document without one.

### E9 · Depth for a novice and for an expert — Learn · Read · Expert

Guidance that helps somebody new gets in an expert's way, so the same content is shown at three
depths rather than written three times. **Learn** keeps every step, including the ones that ask
you to predict first. **Read** shows the answer, the plain words and the real thing. **Expert**
keeps the answer, the relation, the limits and the reference. *(The expertise-reversal effect,
Kalyuga et al. 2003.)*

**Where:** the depth switch at the top of the tool; the page marks blocks `d-l` (Learn only) and
`d-lr` (Learn and Read). **Checked by:** the page generator's test (the plain words are marked,
the predict prompt is Learn-only); the browser walk switches depth and watches the blocks go.

### E10 · A worked example, with real numbers

Show one example fully worked before asking for one. *(The worked-example effect, Renkl et al.
2004.)* A node's first fixture is its worked example — inputs and the answer, from outside this
code.

**Where:** the evidence tab; `sw_central_expectation`, whose three versions are the worked
example of the de-risking record; the example lesson and result in `docs/examples/`.
**Checked by:** the gap pass holds a row with no fixture; the manual's commands run the
examples.

### E11 · Predict before you look

Commit to a guess, then compute, then compare: a wrong guess is the most useful thing on a page.
*(Feynman, "Seeking New Laws"; test-enhanced learning, Roediger & Karpicke 2006.)*

**Where:** the *Try it* step of every node page asks for a prediction before the run, at the
Learn depth. **Checked by:** the page generator's test.

### E12 · Comparisons share an axis; no chart has two

Same axes for every case compared, small multiples over overlays, and never a second y axis.
*(Tufte.)*

**Where:** every figure; the result comparison lists what moved on one scale.
**Checked by:** `tools/panel_check.py` and the stored pictures in `panels/`, which a person
signs (`panels/REVIEW.md`).

### E13 · A "why" ends at a named law or source

A chain of whys stops at something accepted as true, and names it. *(Feynman on why-questions.)*

**Where:** the theory tab ends at the relation and its source; a theory step is derived from the
one before it. **Checked by:** nothing mechanical beyond E3; the physics review (H1b) reads it.

### E14 · Say it without the jargon

A name is not knowledge. The plain-words answer uses no symbol and no term a newcomer would have
to look up; the real thing may, because it is the second step, not the first.

**Where:** `[explain] simply` on the sheet, and *said simply* in the node application's
explanation step. **Checked by:** a person, and `[explain] by` names that person on the page. Words an
assistant drafted from the sheet say so there, for the row's owner to confirm.

### E15 · Do not fool yourself — and do not let the page fool anybody

A gap is shown, not hidden: a row with nothing in it says so, a run says how many rows were
blocked, a result says when a belief it rested on has broken since. *(Rule 5 in `AGENTS.md`;
Feynman, "Cargo Cult Science".)*

**Where:** everywhere. **Checked by:** the gate and the gap pass, and the tests named in
`AGENTS.md`.

---

## Where each surface stands

| surface | E1 | E2 | E3 | E4 | E5 | E8 | E9 | E10 | E11 |
|---|---|---|---|---|---|---|---|---|---|
| node page | Answer first box | first tab | on the claims | breaks + assumptions | if written | tab kinds | yes | evidence tab | Learn |
| `docs/HOW_IT_WORKS.html` | "In short" box per chapter | plain words, how it works, limits | claim tags | "where this picture stops being true" | "a common misreading" | — | yes | "work it yourself" | "guess first" |
| the views — the four layers, Architecture, Inputs, Forms, Results | Answer first box | — | — | — | — | kind on the box | — | — | — |
| run panel and Results | answer, then points | — | — | blocked, weakest factor, broken beliefs | — | — | — | — | — |
| result report (HTML) | first block | Said simply · Where it breaks | derived / declared | Where it breaks | — | section kinds | — | — | — |
| the manual | answer line | — | — | *cannot* table | *when something goes wrong* | section kind | who-filter | examples run | — |
| `docs/*.md`, top-level docs | first block | — | — | — | — | Kind line | — | — | — |
| figures | live answer | question → answer → picture | — | *where the picture stops being true* | — | — | — | — | — |

A dash is a rule that does not fit that surface, not one forgotten.

---

## Adding a surface

A new page, panel or document follows the rules above from the start. Say which in its review;
if a rule does not fit, say why in this file's table rather than leaving a gap nobody chose.

---

## Sources

Rozenblit & Keil 2002 (illusion of explanatory depth); Chi et al. 1994 (self-explanation);
Roediger & Karpicke 2006 (test-enhanced learning); Feynman, *The Character of Physical Law*
(1965), *Fun to Imagine* (1983), "Cargo Cult Science" (1974); Paivio 1986 and Mayer & Fiorella
2014 (dual coding, multimedia principles); Victor 2011 (explorable explanations); Fyfe et al.
2014 (concreteness fading); Shneiderman 1996 (overview first); Tufte 1983/1990 (small
multiples); Brown, C4 model (fixed zoom levels); Procida, Diátaxis (four kinds of
documentation); Minto, the pyramid principle (answer first); Kalyuga et al. 2003
(expertise reversal); Renkl et al. 2004 (worked examples and fading); Schwartz & Bransford 1998
(contrasting cases); Tippett 2010 (refutation text); Gentner 1983 (structure mapping). All as
cited in *Eight Loops, One Beam v2*, which this standard is drawn from.
