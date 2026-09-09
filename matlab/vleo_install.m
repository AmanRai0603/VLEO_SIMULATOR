function vleo_install()
%VLEO_INSTALL  Pin the interpreter, set out-of-process execution, verify.
%
%   MATLAB supports a specific range of interpreter versions that differs by
%   release, and a user on another release cannot load the wheel and reports the
%   tool as broken. So the version is pinned here and checked, once, rather than
%   discovered by a stack trace.
%
%   Out of process by default: MATLAB ships its own runtime libraries, and an
%   in-process native extension can bring MATLAB down.
    supported = {'3.11', '3.12'};
    e = pyenv;
    if ~ismember(char(e.Version), supported)
        error('vleo:interpreter', ...
            ['This MATLAB release loads Python %s. The wheel is built for %s.\n' ...
             'Set it with: pyenv(''Version'', ''<path to python%s>'')'], ...
            char(e.Version), strjoin(supported, ' or '), supported{1});
    end
    if e.ExecutionMode ~= "OutOfProcess"
        pyenv('ExecutionMode', 'OutOfProcess');
    end

    v = py.vleo.version();
    fprintf('vleo  kernel %s  graph %s  %d nodes\n', ...
            char(v{1}), char(v{2}), double(v{3}));

    % Verify against a golden vector on the user's own machine, so a person can
    % prove their install is sound unaided.
    r = vleo.evaluate('orbit_velocity', 'set', struct('orbit_altitude', 250e3));
    expected = 7754.84549737;
    err = abs(r.value - expected) / expected;
    if err > 1e-9
        error('vleo:selftest', ...
            ['Golden vector disagrees: got %.9f, expected %.9f (relative %.2e).\n' ...
             'This is a determinism fault, not a physics one. Report the kernel ' ...
             'hash %s and the platform.'], r.value, expected, err, r.kernel);
    end
    fprintf('selftest ok — orbit_velocity at 250 km agrees to %.1e\n', err);
end
