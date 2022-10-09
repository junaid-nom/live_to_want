
# Model notes

### BERT: 
 - Just the encoding attention part basically.
 - But does "Narrow attention" where the heads are splitting the input
 - Also seems to have 1 huge K Q V that is multiplied by the entire input instead of the by-token KQV?

### Albert:
 - Like BERT but smaller embedding size for each word and instead of N attention layers has 1 attention layer applied multiple times.
 - The savings are then used to make the hidden layer embeddings 10-20x bigger.

### BART:
 - Uses BERT encoder and GPT decoder to denoise stuff.

### RoBERTa:
 - Optimized BERT. Larger batch size, different autotrain task (find wrong word) and changing wrong word every epoch of training

### DeBERTa:
 - Separates position vector and token embedding. Positions are relative -2-2 (clamped, so -3 pos becomes -2)
   - Has 3 KQV one for each combo of Inp-Pos, Pos-Inp, Inp-Inp. Inp here means the token embedding without the pos.
   - Attention: Inp,Pos -> Inp. Then for subsequent attention layers add pos back in separately. so Layer2 is still inp2,pos -> inp3
   - BUT they say you also need ABSOLUTE pos encoding? Relative lets you essentially weight share, but absolute also still necessary.
     - In the very last layer it has absolute layer. Found absolute pos at end is better than early. 
       - Seems dumb. Maybe should have both rel and abs pos always?
   - There scaled up model actually combines the KQV for inp and pos. Which kinda is reverting back to not disentangling them, but they are still readding the pos every layer so maybe that helps.

### DistilBert:
 - 40% the size of BERT?

### Electra:
 - In ELECTRA, instead of masking the input, the approach corrupts it by replacing some input tokens with plausible alternatives sampled from a small generator network. Then, instead of training a model that predicts the original identities of the corrupted tokens, a discriminative model is trained that predicts whether each token in the corrupted input was replaced by a generator sample or not. This new pre-training task is more efficient than MLM because the model learns from all input tokens rather than just the small subset that was masked out.
   - Needs to use a Generator G to fool the encoder like a GAN
     - G generator is literally just BERT trained on MLM. And the other network basically is just going "Is this replaced word really the original output or is it G's attempt?)
  
### Marian:
Not really sure what this is doing different? Seems to just be a more efficient super optimized microsoft implementation of BERT stuff?

### Mobile BERT:
Smaller BERT that is trained via teacher model from the BERT_LARGE model
 - They train the net so that each layer mimics the output of BERT-LARGE one by one. They freeze layers below the layer they are currently training so its sequentially training them.

### T5:
Seems they just trained on a HYUGE dataset? The other big thing is they reframe every problem into a sort of question -> text response. "Translate english to german: that is good" -> "das ist gut"

### XLNET:
GPT uses autoreggressive strategy where coming from one direction it predicts the next word from past words (can also go backwards use later words predicting earlier words).
But BERT is bidirectional since it sees the entire sentence at the same time. But the issue with BERT is when there is more than one masked word, it should predict
the 2nd masked word based on the first masked word prediction as well, but it doesn't since it has to output them at the same time instead of one at a time like autoreggressive GPT.
It does this by kind of randomizing the order of the sentence and running it with a bunch of random permutations as training data so that it eventually will see every word "before" the masked word in some permutation. Something vaguely like that.

### GPT:
Just a decoder.

### Reformer
Upgrades BERT to be able to handle millions of tokens instead of just 512. Does this using LSH algo.
Basically it finds Q=K is fine. Then puts all tokens into a bucket, based on how similar their Qs are. Basically going, instead of checking every token's attention worthyness, just check if this bucket is worthy and that means everything in this bucket is high value.
It then does "local attention" (instead of entire input look at just a chunk of it at a time) but uses the LSH to figure out what should be in the chunk? Not sure.
It also chunks linear layers to only run parts of them at a time? IDK
It deals with gigantic positional numbers by adding dimensions to it and multiplying then. 49 = 7x7. So you only need 14 one-hot bits instead of 49? Not sure.

### ProphetNet
Instead of predicting the next word. It predicts the next N words at once.

### Longformer
For very long sequences, it uses a local window. But it augments this window by having special "global" tokens that will be attended to by every token, and will also weirdly when its there turn, attend to every token in the sequence. Not sure how that latter is achieved tho.

### Pegasus
Pretrains on summerization by masking sentences and then it has to output all of them based on remaining sentences?

### GPT NEO
The architecture is similar to GPT2 except that GPT Neo uses local attention in every other layer with a window size of 256 tokens.

### mBART
mBART is one of the first methods for pre-training a complete sequence-to-sequence model by denoising full texts in multiple languages

### M2M100
A translator transformer but its trained on directly translating languages from one to another without using english as an intermediate.

### FNET:
WAIT ARE YOU SHITTING ME.
ATTENTION WAS BS ALL ALONG. ITS JUST WEIGHT SHARING LIKE CONVOLUTION BUT OVER THE ENTIRE SENTENCE.
PUTTING 2 LINEAR LAYERS IN PLACE OF ATTENTION LAYER IS BASICALLY JUST AS GOOD AS ATTENTION AND 6-8x FASTER!
https://youtu.be/JJR3pBl78zw






