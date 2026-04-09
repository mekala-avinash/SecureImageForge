import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import axios from 'axios';
import { ToggleLeft, ToggleRight, Info, Sparkle } from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const EnhancedNewBuild = () => {
  const navigate = useNavigate();
  const [advancedMode, setAdvancedMode] = useState(false);
  const [runtimeVersions, setRuntimeVersions] = useState({});
  const [baseImageTags, setBaseImageTags] = useState({});
  const [cisLevels, setCisLevels] = useState({});
  const [sbomOptions, setSbomOptions] = useState({});
  
  const [formData, setFormData] = useState({
    name: '',
    runtime: 'java',
    base_image: 'alpine',
    compliance_profiles: ['cis'],
    architecture: ['amd64'],
    remove_shell: true,
    remove_package_manager: true,
    enable_sbom: true,
    enable_signing: true,
    // Advanced fields
    runtime_version: null,
    runtime_distribution: null,
    base_image_tag: null,
    binary_whitelist: [],
    env_sanitization_rules: [],
    cis_level: 1,
    fips_mode_enabled: false,
    custom_labels: {},
    sbom_format: 'cyclonedx',
    sbom_scan_depth: 'os_and_runtime'
  });
  
  const [submitting, setSubmitting] = useState(false);
  const [labelInput, setLabelInput] = useState({ key: '', value: '' });
  const [binaryInput, setBinaryInput] = useState('');
  const [envInput, setEnvInput] = useState('');

  useEffect(() => {
    fetchConfigurations();
  }, []);

  useEffect(() => {
    // Auto-select default version when runtime changes
    if (runtimeVersions[formData.runtime]) {
      const defaultVer = runtimeVersions[formData.runtime].default_version;
      const defaultDist = runtimeVersions[formData.runtime].default_distribution;
      setFormData(prev => ({
        ...prev,
        runtime_version: defaultVer,
        runtime_distribution: defaultDist
      }));
    }
  }, [formData.runtime, runtimeVersions]);

  const fetchConfigurations = async () => {
    try {
      const [versionsRes, tagsRes, cisRes, sbomRes] = await Promise.all([
        axios.get(`${API}/runtime-versions`),
        axios.get(`${API}/base-image-tags`),
        axios.get(`${API}/cis-levels`),
        axios.get(`${API}/sbom-formats`)
      ]);
      
      setRuntimeVersions(versionsRes.data.runtimes);
      setBaseImageTags(tagsRes.data.base_images);
      setCisLevels(cisRes.data.levels);
      setSbomOptions(sbomRes.data);
    } catch (error) {
      console.error('Error fetching configurations:', error);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setSubmitting(true);

    try {
      const response = await axios.post(`${API}/builds`, formData);
      navigate(`/builds/${response.data.id}`);
    } catch (error) {
      console.error('Error creating build:', error);
      alert('Failed to create build: ' + (error.response?.data?.detail || error.message));
    } finally {
      setSubmitting(false);
    }
  };

  const toggleCompliance = (profile) => {
    setFormData(prev => ({
      ...prev,
      compliance_profiles: prev.compliance_profiles.includes(profile)
        ? prev.compliance_profiles.filter(p => p !== profile)
        : [...prev.compliance_profiles, profile]
    }));
  };

  const toggleArchitecture = (arch) => {
    setFormData(prev => ({
      ...prev,
      architecture: prev.architecture.includes(arch)
        ? prev.architecture.filter(a => a !== arch)
        : [...prev.architecture, arch]
    }));
  };

  const addCustomLabel = () => {
    if (labelInput.key && labelInput.value) {
      setFormData(prev => ({
        ...prev,
        custom_labels: {
          ...prev.custom_labels,
          [labelInput.key]: labelInput.value
        }
      }));
      setLabelInput({ key: '', value: '' });
    }
  };

  const removeLabel = (key) => {
    setFormData(prev => {
      const newLabels = { ...prev.custom_labels };
      delete newLabels[key];
      return { ...prev, custom_labels: newLabels };
    });
  };

  const addBinaryToWhitelist = () => {
    if (binaryInput && !formData.binary_whitelist.includes(binaryInput)) {
      setFormData(prev => ({
        ...prev,
        binary_whitelist: [...prev.binary_whitelist, binaryInput]
      }));
      setBinaryInput('');
    }
  };

  const removeBinary = (binary) => {
    setFormData(prev => ({
      ...prev,
      binary_whitelist: prev.binary_whitelist.filter(b => b !== binary)
    }));
  };

  const addEnvRule = () => {
    if (envInput && !formData.env_sanitization_rules.includes(envInput)) {
      setFormData(prev => ({
        ...prev,
        env_sanitization_rules: [...prev.env_sanitization_rules, envInput]
      }));
      setEnvInput('');
    }
  };

  const removeEnvRule = (rule) => {
    setFormData(prev => ({
      ...prev,
      env_sanitization_rules: prev.env_sanitization_rules.filter(r => r !== rule)
    }));
  };

  const getAvailableVersions = () => {
    return runtimeVersions[formData.runtime]?.versions || {};
  };

  const getAvailableDistributions = () => {
    const versions = getAvailableVersions();
    if (!formData.runtime_version || !versions[formData.runtime_version]) return {};
    return versions[formData.runtime_version].distributions || {};
  };

  const getAvailableTags = () => {
    return baseImageTags[formData.base_image]?.tags || {};
  };

  const isFIPSSupported = () => {
    const dists = getAvailableDistributions();
    const dist = dists[formData.runtime_distribution];
    return dist?.fips_supported || false;
  };

  return (
    <div className="max-w-4xl mx-auto px-6 sm:px-8 py-8" data-testid="enhanced-new-build-page">
      <div className="mb-8">
        <div className="flex items-center justify-between mb-2">
          <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter" style={{fontFamily: 'Chivo'}}>
            New Build Configuration
          </h2>
          <button
            type="button"
            onClick={() => setAdvancedMode(!advancedMode)}
            className={`flex items-center gap-2 px-4 py-2 rounded-sm border transition-all ${
              advancedMode
                ? 'bg-[#002FA7] text-white border-[#002FA7]'
                : 'bg-white text-[#4B5563] border-black/20 hover:border-[#002FA7]'
            }`}
            data-testid="toggle-advanced-mode"
          >
            {advancedMode ? <ToggleRight size={20} weight="fill" /> : <ToggleLeft size={20} />}
            <span className="text-sm font-medium uppercase tracking-wider">
              {advancedMode ? 'Advanced Mode' : 'Simple Mode'}
            </span>
            <Sparkle size={16} weight={advancedMode ? 'fill' : 'regular'} />
          </button>
        </div>
        <p className="text-base text-[#4B5563]">
          {advancedMode 
            ? 'Full enterprise controls with granular runtime, OS, and compliance settings'
            : 'Quick setup with recommended defaults - toggle Advanced Mode for detailed control'}
        </p>
      </div>

      <form onSubmit={handleSubmit} className="bg-white border border-black/10 rounded-sm p-8">
        {/* Build Name */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2" htmlFor="name">
            Build Name *
          </label>
          <input
            id="name"
            type="text"
            value={formData.name}
            onChange={(e) => setFormData({...formData, name: e.target.value})}
            className="w-full border border-black/20 rounded-sm px-4 py-2 focus:ring-2 focus:ring-[#002FA7]/30 focus:border-[#002FA7] outline-none transition-all bg-white font-mono text-sm"
            placeholder="my-secure-app"
            required
            data-testid="input-build-name"
          />
        </div>

        {/* Runtime - Enhanced in Advanced Mode */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Runtime Environment *
          </label>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            {['java', 'dotnet', 'go', 'nodejs'].map((runtime) => (
              <button
                key={runtime}
                type="button"
                onClick={() => setFormData({...formData, runtime})}
                className={`p-4 border rounded-sm text-left transition-all ${
                  formData.runtime === runtime
                    ? 'border-[#002FA7] bg-[#002FA7]/5'
                    : 'border-black/10 hover:border-black/30'
                }`}
                data-testid={`runtime-${runtime}`}
              >
                <div className="text-sm font-medium uppercase tracking-wider">{runtime}</div>
              </button>
            ))}
          </div>
        </div>

        {/* Advanced: Runtime Version & Distribution */}
        {advancedMode && (
          <div className="mb-6 p-4 bg-[#002FA7]/5 border border-[#002FA7]/20 rounded-sm">
            <div className="flex items-center gap-2 mb-3">
              <Info size={16} className="text-[#002FA7]" />
              <h4 className="text-sm font-bold uppercase tracking-wider">Runtime Details</h4>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              {/* Version Selector */}
              <div>
                <label className="block text-xs uppercase tracking-wider font-medium mb-2">Version</label>
                <select
                  value={formData.runtime_version || ''}
                  onChange={(e) => setFormData({...formData, runtime_version: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
                  data-testid="select-runtime-version"
                >
                  {Object.entries(getAvailableVersions()).map(([ver, data]) => (
                    <option key={ver} value={ver}>
                      {ver} {data.lts ? '(LTS)' : ''} {data.recommended ? '⭐' : ''}
                    </option>
                  ))}
                </select>
              </div>

              {/* Distribution Selector */}
              <div>
                <label className="block text-xs uppercase tracking-wider font-medium mb-2">Distribution</label>
                <select
                  value={formData.runtime_distribution || ''}
                  onChange={(e) => setFormData({...formData, runtime_distribution: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
                  data-testid="select-runtime-distribution"
                >
                  {Object.entries(getAvailableDistributions()).map(([dist, data]) => (
                    <option key={dist} value={dist}>
                      {dist} {data.fips_supported ? '(FIPS)' : ''}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* FIPS Mode Toggle */}
            {isFIPSSupported() && (
              <div className="mt-3 p-3 bg-white rounded-sm">
                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.fips_mode_enabled}
                    onChange={(e) => setFormData({...formData, fips_mode_enabled: e.target.checked})}
                    className="w-4 h-4 accent-[#002FA7]"
                    data-testid="toggle-fips-mode"
                  />
                  <span className="text-sm font-medium">Enable FIPS 140-2 Compliant Cryptography</span>
                </label>
              </div>
            )}
          </div>
        )}

        {/* Base Image */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Base Image *
          </label>
          <div className="grid grid-cols-3 gap-3">
            {['alpine', 'debian', 'distroless'].map((base) => (
              <button
                key={base}
                type="button"
                onClick={() => setFormData({...formData, base_image: base})}
                className={`p-4 border rounded-sm text-center transition-all ${
                  formData.base_image === base
                    ? 'border-[#002FA7] bg-[#002FA7]/5'
                    : 'border-black/10 hover:border-black/30'
                }`}
                data-testid={`base-${base}`}
              >
                <div className="text-sm font-medium uppercase tracking-wider">{base}</div>
              </button>
            ))}
          </div>
        </div>

        {/* Advanced: Base Image Tag */}
        {advancedMode && (
          <div className="mb-6 p-4 bg-[#002FA7]/5 border border-[#002FA7]/20 rounded-sm">
            <div className="flex items-center gap-2 mb-3">
              <Info size={16} className="text-[#002FA7]" />
              <h4 className="text-sm font-bold uppercase tracking-wider">Base Image Tag</h4>
            </div>
            
            <select
              value={formData.base_image_tag || ''}
              onChange={(e) => setFormData({...formData, base_image_tag: e.target.value})}
              className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
              data-testid="select-base-tag"
            >
              <option value="">Auto-select recommended tag</option>
              {Object.entries(getAvailableTags()).map(([tag, data]) => (
                <option key={tag} value={tag}>
                  {tag} {data.security_status === 'latest' && '⭐'} ({data.security_status})
                </option>
              ))}
            </select>
            <p className="text-xs text-[#4B5563] mt-2">Pin to specific tag to prevent breaking changes from upstream updates</p>
          </div>
        )}

        {/* Architecture */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Target Architecture *
          </label>
          <div className="grid grid-cols-2 gap-3">
            {['amd64', 'arm64'].map((arch) => (
              <button
                key={arch}
                type="button"
                onClick={() => toggleArchitecture(arch)}
                className={`p-4 border rounded-sm text-center transition-all ${
                  formData.architecture.includes(arch)
                    ? 'border-[#002FA7] bg-[#002FA7]/5'
                    : 'border-black/10 hover:border-black/30'
                }`}
                data-testid={`arch-${arch}`}
              >
                <div className="text-sm font-medium uppercase tracking-wider">{arch}</div>
                {formData.architecture.length > 1 && formData.architecture.includes(arch) && (
                  <div className="text-xs mt-1 text-[#002FA7]">Multi-arch</div>
                )}
              </button>
            ))}
          </div>
        </div>

        {/* Compliance Profiles */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Compliance Profiles *
          </label>
          <div className="grid grid-cols-3 gap-3">
            {['hipaa', 'soc2', 'cis'].map((profile) => (
              <button
                key={profile}
                type="button"
                onClick={() => toggleCompliance(profile)}
                className={`p-4 border rounded-sm text-center transition-all ${
                  formData.compliance_profiles.includes(profile)
                    ? 'border-[#002FA7] bg-[#002FA7]/5'
                    : 'border-black/10 hover:border-black/30'
                }`}
                data-testid={`compliance-${profile}`}
              >
                <div className="text-xs font-medium uppercase tracking-wider">{profile}</div>
              </button>
            ))}
          </div>
        </div>

        {/* Advanced: CIS Level */}
        {advancedMode && formData.compliance_profiles.includes('cis') && (
          <div className="mb-6 p-4 bg-[#002FA7]/5 border border-[#002FA7]/20 rounded-sm">
            <div className="flex items-center gap-2 mb-3">
              <Info size={16} className="text-[#002FA7]" />
              <h4 className="text-sm font-bold uppercase tracking-wider">CIS Benchmark Level</h4>
            </div>
            
            <div className="grid grid-cols-2 gap-3">
              {[1, 2].map((level) => {
                const levelData = cisLevels[`level${level}`] || {};
                return (
                  <button
                    key={level}
                    type="button"
                    onClick={() => setFormData({...formData, cis_level: level})}
                    className={`p-3 border rounded-sm text-left transition-all ${
                      formData.cis_level === level
                        ? 'border-[#002FA7] bg-white'
                        : 'border-black/10 hover:border-black/30'
                    }`}
                    data-testid={`cis-level-${level}`}
                  >
                    <div className="font-bold text-sm">Level {level}</div>
                    <div className="text-xs text-[#4B5563] mt-1">{levelData.description}</div>
                  </button>
                );
              })}
            </div>
          </div>
        )}

        {/* Basic Hardening Options (always visible) */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-3">
            Hardening Options
          </label>
          <div className="space-y-3">
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={formData.remove_shell}
                onChange={(e) => setFormData({...formData, remove_shell: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
                data-testid="option-remove-shell"
              />
              <span className="text-sm">Remove shell binaries (sh/bash)</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={formData.remove_package_manager}
                onChange={(e) => setFormData({...formData, remove_package_manager: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
                data-testid="option-remove-pkg-mgr"
              />
              <span className="text-sm">Remove package managers (apt/apk)</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={formData.enable_sbom}
                onChange={(e) => setFormData({...formData, enable_sbom: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
                data-testid="option-enable-sbom"
              />
              <span className="text-sm">Generate SBOM</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={formData.enable_signing}
                onChange={(e) => setFormData({...formData, enable_signing: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
                data-testid="option-enable-signing"
              />
              <span className="text-sm">Sign image with Cosign</span>
            </label>
          </div>
        </div>

        {/* Advanced: Binary Whitelist */}
        {advancedMode && formData.remove_shell && (
          <div className="mb-6 p-4 bg-yellow-50 border border-yellow-200 rounded-sm">
            <div className="flex items-center gap-2 mb-3">
              <Info size={16} className="text-yellow-700" />
              <h4 className="text-sm font-bold uppercase tracking-wider text-yellow-700">Binary Whitelist ("Break Glass")</h4>
            </div>
            <p className="text-xs text-yellow-700 mb-3">Binaries to preserve even when hardening is enabled (e.g., for monitoring agents)</p>
            
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={binaryInput}
                onChange={(e) => setBinaryInput(e.target.value)}
                placeholder="/usr/bin/curl"
                className="flex-1 border border-yellow-300 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-yellow-500/30 outline-none"
                data-testid="input-binary-whitelist"
              />
              <button
                type="button"
                onClick={addBinaryToWhitelist}
                className="px-4 py-2 bg-yellow-600 text-white rounded-sm text-sm hover:bg-yellow-700"
              >
                Add
              </button>
            </div>
            
            {formData.binary_whitelist.length > 0 && (
              <div className="space-y-1">
                {formData.binary_whitelist.map((binary) => (
                  <div key={binary} className="flex items-center justify-between bg-white px-3 py-2 rounded-sm text-sm font-mono">
                    <span>{binary}</span>
                    <button
                      type="button"
                      onClick={() => removeBinary(binary)}
                      className="text-red-600 hover:text-red-800 text-xs"
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Advanced: Environment Variable Sanitization */}
        {advancedMode && (
          <div className="mb-6 p-4 bg-[#002FA7]/5 border border-[#002FA7]/20 rounded-sm">
            <div className="flex items-center gap-2 mb-3">
              <Info size={16} className="text-[#002FA7]" />
              <h4 className="text-sm font-bold uppercase tracking-wider">Environment Variable Sanitization</h4>
            </div>
            <p className="text-xs text-[#4B5563] mb-3">Environment variables to strip/mask during build (prevent secret leakage)</p>
            
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={envInput}
                onChange={(e) => setEnvInput(e.target.value)}
                placeholder="AWS_SECRET_KEY"
                className="flex-1 border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
                data-testid="input-env-sanitization"
              />
              <button
                type="button"
                onClick={addEnvRule}
                className="px-4 py-2 bg-[#002FA7] text-white rounded-sm text-sm hover:bg-[#002FA7]/90"
              >
                Add
              </button>
            </div>
            
            {formData.env_sanitization_rules.length > 0 && (
              <div className="space-y-1">
                {formData.env_sanitization_rules.map((rule) => (
                  <div key={rule} className="flex items-center justify-between bg-white px-3 py-2 rounded-sm text-sm font-mono">
                    <span>{rule}</span>
                    <button
                      type="button"
                      onClick={() => removeEnvRule(rule)}
                      className="text-red-600 hover:text-red-800 text-xs"
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Advanced: Custom Labels */}
        {advancedMode && (
          <div className="mb-6 p-4 bg-[#002FA7]/5 border border-[#002FA7]/20 rounded-sm">
            <div className="flex items-center gap-2 mb-3">
              <Info size={16} className="text-[#002FA7]" />
              <h4 className="text-sm font-bold uppercase tracking-wider">Custom Docker Labels</h4>
            </div>
            <p className="text-xs text-[#4B5563] mb-3">Add metadata for asset tracking (owner, department, cost-center)</p>
            
            <div className="grid grid-cols-2 gap-2 mb-2">
              <input
                type="text"
                value={labelInput.key}
                onChange={(e) => setLabelInput({...labelInput, key: e.target.value})}
                placeholder="owner"
                className="border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
                data-testid="input-label-key"
              />
              <input
                type="text"
                value={labelInput.value}
                onChange={(e) => setLabelInput({...labelInput, value: e.target.value})}
                placeholder="team-platform"
                className="border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
                data-testid="input-label-value"
              />
            </div>
            <button
              type="button"
              onClick={addCustomLabel}
              className="w-full px-4 py-2 bg-[#002FA7] text-white rounded-sm text-sm hover:bg-[#002FA7]/90"
            >
              Add Label
            </button>
            
            {Object.keys(formData.custom_labels).length > 0 && (
              <div className="mt-3 space-y-1">
                {Object.entries(formData.custom_labels).map(([key, value]) => (
                  <div key={key} className="flex items-center justify-between bg-white px-3 py-2 rounded-sm text-sm font-mono">
                    <span>{key}: {value}</span>
                    <button
                      type="button"
                      onClick={() => removeLabel(key)}
                      className="text-red-600 hover:text-red-800 text-xs"
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Advanced: SBOM Configuration */}
        {advancedMode && formData.enable_sbom && (
          <div className="mb-6 p-4 bg-[#002FA7]/5 border border-[#002FA7]/20 rounded-sm">
            <div className="flex items-center gap-2 mb-3">
              <Info size={16} className="text-[#002FA7]" />
              <h4 className="text-sm font-bold uppercase tracking-wider">SBOM Configuration</h4>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-xs uppercase tracking-wider font-medium mb-2">Format</label>
                <select
                  value={formData.sbom_format}
                  onChange={(e) => setFormData({...formData, sbom_format: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
                  data-testid="select-sbom-format"
                >
                  {Object.entries(sbomOptions.formats || {}).map(([key, data]) => (
                    <option key={key} value={key}>{data.name} ({data.version})</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs uppercase tracking-wider font-medium mb-2">Scan Depth</label>
                <select
                  value={formData.sbom_scan_depth}
                  onChange={(e) => setFormData({...formData, sbom_scan_depth: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono focus:ring-2 focus:ring-[#002FA7]/30 outline-none"
                  data-testid="select-sbom-depth"
                >
                  {Object.entries(sbomOptions.scan_depths || {}).map(([key, data]) => (
                    <option key={key} value={key}>{data.name}</option>
                  ))}
                </select>
              </div>
            </div>
          </div>
        )}

        {/* Submit */}
        <div className="flex gap-4 pt-4 border-t border-black/10">
          <button
            type="submit"
            disabled={submitting}
            className="btn-primary flex-1"
            data-testid="submit-build-btn"
          >
            {submitting ? 'Creating Build...' : 'Start Build'}
          </button>
          <button
            type="button"
            onClick={() => navigate('/builds')}
            className="btn-secondary"
            data-testid="cancel-build-btn"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
};
