## Equations

```
S_f107(L) = 1 - MSE_outlook(L) / MSE_persistence(L)
```

## Derivation

A design that keys off a published forecast is exposed to the forecast being worthless, and 'worthless' has a precise meaning: no better than the cheapest thing you could have done instead. That makes the question comparative rather than absolute. A skill score is the standard form of the comparison — one minus the ratio of mean squared errors — and its entire content is the baseline it is measured against, which is why the baseline is declared on the sheet rather than left to whoever reads the number.

1. Define skill as the fraction of a baseline's mean squared error that the forecast removes. One is perfect, zero is no better than the baseline, negative is worse than not bothering. `S(L) = 1 - MSE_outlook(L) / MSE_baseline(L)`
2. Which makes the choice of baseline the whole measurement, not a detail of it. Persistence is the right baseline here because it is what a design would do with no forecast at all: assume today's flux continues.
3. Persistence needs a definition of 'today', and this is where a verification leaks. The baseline is the last observation STRICTLY BEFORE the issue date. Allowing the observation ON the issue date gives the baseline a number the forecaster did not have — and for the 719 issues indexed from lead 0, that observation is itself a forecast target.
4. The leak reverses a conclusion rather than shading one. With the leaky baseline the lead-1 skill reads -1.314 and the outlook appears to lose to persistence through lead 4; with the strict baseline it reads +0.069 and the outlook wins from lead 1. `lead 1: strict +0.0685 leaky -1.3138`
5. Measured strictly at every verifiable lead, the curve rises, peaks and then falls through zero. Skill is positive from lead 1 to lead 23, peaks at +0.438 at lead 9, and is negative at 24, 25 and 26 — against samples of 866 to 868 pairs each, so the negative tail is not a small-sample artefact. `lead 9: +0.4375 lead 23: +0.0177 lead 26: -0.0221`
6. The interior maximum is why the row is a table and not a fit: a monotone curve through these points would be a different claim about predictability than the record makes.
7. And the answer at the declared lead is the design-relevant one. At the far end of its own published window the outlook is very slightly worse than assuming nothing changes, so a design keying off the end of the outlook is attending to a forecast that has stopped carrying information.

## Assumptions

- Persistence is the last observation STRICTLY BEFORE the issue date, and this choice decides the answer. Fails when: the baseline is allowed the observation on the issue date itself. 719 of the 1281 issues index their rows from lead 0, so the issue date IS a forecast target for most of the record, and handing it to the baseline gives persistence a number the forecaster did not have. The whole short-lead conclusion turns on it: the same arithmetic then reports -1.314 at lead 1 instead of +0.069, and the outlook appears to lose to persistence through lead 4 when it does not. A skill score is a statement about a baseline, so the baseline is declared here rather than left to whoever reads the number
- It goes negative at the far end of the window, and that is the answer, not a defect. Fails when: the last three verifiable leads are read as noise. Skill is positive from lead 1 through lead 23, peaks at +0.438 at lead 9, and is negative at leads 24, 25 and 26 — -0.036, -0.032 and -0.022 against samples of 866 to 868 pairs each. At the far end of its own published window the outlook is very slightly worse than assuming nothing changes. A design keying off the end of the outlook is paying attention to a forecast that has stopped carrying information
- Skill against persistence is not accuracy. Fails when: a positive score is read as the forecast being good. The outlook's own RMS error grows from 10.3 sfu at lead 1 to 24.9 sfu at lead 26; what improves through the middle leads is only its ratio to a baseline that degrades faster. Peak skill of +0.438 at lead 9 sits on an RMS error of 21.2 sfu, which is 18% of a typical F10.7. The forecast is never accurate in the window; it is merely better than nothing for most of it
- One score over 29 years, pooled across cycles. Fails when: the skill is activity-dependent, which it will be: persistence is a strong baseline in a quiet Sun and a weak one in a rising cycle, so pooling cycles 23, 24 and the rise of 25 averages over regimes where the comparison means different things. The sample is 866 to 1257 pairs per lead and is not conditioned on phase. A phase-conditioned skill would be a separate row and would need the epoch, which now exists

## Validity

From -0.1 to 0.5 One. Below: the worst measured skill is -0.036, at lead 24. A bound at -0.1 leaves room for the three negative leads and refuses anything that would say the published outlook is substantially worse than doing nothing, which the record does not support. Above: the best measured skill is +0.438, at lead 9. A skill above 0.5 against persistence would mean the outlook halves the baseline's mean squared error, and nothing in this record comes close; an answer there means the table was misread or the bundle changed underneath it.

Skill against persistence is not accuracy. The outlook's own RMS error grows from 10.3 sfu at lead 1 to 24.9 at lead 26; what improves through the middle leads is only its ratio to a baseline degrading faster. Peak skill of +0.438 at lead 9 sits on an RMS error of 21.2 sfu, about 18 per cent of a typical F10.7 — so the forecast is never accurate in this window, merely better than nothing for most of it. The score is also pooled over 29 years without conditioning on activity, and persistence is a strong baseline in a quiet Sun and a weak one in a rising cycle.
