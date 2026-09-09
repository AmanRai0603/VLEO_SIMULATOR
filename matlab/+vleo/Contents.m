% VLEO — the MATLAB facade.
%
%   GENERATED from the same declarations as every other binding. A user of this
%   package never types the word Python: the wheel is an implementation detail
%   of how MATLAB reaches the engine, not something to configure per script.
%
%   vleo_install    pins the interpreter, sets out-of-process execution and
%                   verifies the install against golden vectors. One command,
%                   and a clean MATLAB comes up working.
%   vleo.evaluate   one node, one case, one answer with its provenance
%   vleo.sweep      one call, a table back — never a loop calling evaluate per
%                   point, which is slow for reasons that are MATLAB's and gets
%                   blamed on the engine
%   vleo.version    kernel, graph and node count; a mismatch against the daemon
%                   warns loudly rather than disagreeing silently
%
%   MATLAB's job here is fixtures. Explore, produce a golden vector, and commit
%   that vector as what the Rust must reproduce.
