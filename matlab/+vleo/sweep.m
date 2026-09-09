function t = sweep(node, over, from, to, varargin)
%SWEEP  One call, a table back.
%
%   t = vleo.sweep('prop_thrust_to_drag', 'orbit_altitude', 150e3, 450e3)
%
%   One call rather than five hundred. The marshalling cost per call is what
%   makes a loop calling evaluate per point slow, and the engine gets blamed.
%
%   Refused points appear in t.Properties.UserData.refused rather than being
%   dropped: a sweep in which some rows quietly used a substituted value is a
%   sweep whose conclusion is unknown.
    p = inputParser;
    addParameter(p, 'points', 64);
    addParameter(p, 'case', 'nominal');
    parse(p, varargin{:});

    out = py.vleo.sweep(node, over, from, to, ...
                        pyargs('points', int32(p.Results.points), ...
                               'case', p.Results.case));
    x = double(py.array.array('d', out{1}));
    y = double(py.array.array('d', out{2}));
    t = table(x(:), y(:), 'VariableNames', {over, node});
    t.Properties.UserData.refused = out{3};
end
