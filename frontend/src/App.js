import React, { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Link, useNavigate } from 'react-router-dom';
import axios from 'axios';
import '@/App.css';
import { 
  Cube, 
  ShieldCheck, 
  Bug, 
  ChartBar, 
  List, 
  Plus,
  CheckCircle,
  XCircle,
  Clock,
  Warning,
  ShieldWarning,
  Detective,
  Gear
} from '@phosphor-icons/react';
import { BuildDetail } from './components/BuildDetail';
import { BuildsList } from './components/BuildsList';
import { Analytics } from './components/Analytics';
import { Policies } from './components/Policies';
import { EnhancedNewBuild } from './components/EnhancedNewBuild';
import { Exceptions } from './components/Exceptions';
import { DriftDetection } from './components/DriftDetection';
import { RemediationPolicies } from './components/RemediationPolicies';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

// Header Component
const Header = () => {
  return (
    <header className="forge-header" data-testid="main-header">
      <div className="max-w-7xl mx-auto px-6 sm:px-8 py-4 flex items-center justify-between">
        <Link to="/" className="flex items-center gap-3" data-testid="logo-link">
          <Cube size={32} weight="bold" className="text-[#002FA7]" />
          <div>
            <h1 className="text-xl font-bold tracking-tight" style={{fontFamily: 'Chivo'}}>SecureImage Forge</h1>
            <p className="text-xs uppercase tracking-wider text-[#4B5563]">Enterprise Image Hardening</p>
          </div>
        </Link>
        
        <nav className="flex items-center gap-6">
          <Link 
            to="/" 
            className="text-sm font-medium hover:text-[#002FA7] transition-colors"
            data-testid="nav-dashboard"
          >
            Dashboard
          </Link>
          <Link 
            to="/builds" 
            className="text-sm font-medium hover:text-[#002FA7] transition-colors"
            data-testid="nav-builds"
          >
            Builds
          </Link>
          <Link 
            to="/analytics" 
            className="text-sm font-medium hover:text-[#002FA7] transition-colors"
            data-testid="nav-analytics"
          >
            Analytics
          </Link>
          <Link 
            to="/policies" 
            className="text-sm font-medium hover:text-[#002FA7] transition-colors"
            data-testid="nav-policies"
          >
            Policies
          </Link>
          <Link 
            to="/exceptions" 
            className="text-sm font-medium hover:text-[#002FA7] transition-colors"
            data-testid="nav-exceptions"
          >
            Exceptions
          </Link>
          <Link 
            to="/drift" 
            className="text-sm font-medium hover:text-[#002FA7] transition-colors"
            data-testid="nav-drift"
          >
            Drift
          </Link>
          <Link 
            to="/remediation-policies" 
            className="text-sm font-medium hover:text-[#002FA7] transition-colors"
            data-testid="nav-remediation-policies"
          >
            Auto-Fix
          </Link>
          <Link 
            to="/new" 
            className="btn-primary flex items-center gap-2"
            data-testid="nav-new-build"
          >
            <Plus size={16} weight="bold" />
            New Build
          </Link>
        </nav>
      </div>
    </header>
  );
};

// Dashboard Component
const Dashboard = () => {
  const [stats, setStats] = useState(null);
  const [recentBuilds, setRecentBuilds] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchDashboardData();
  }, []);

  const fetchDashboardData = async () => {
    try {
      const [statsRes, buildsRes] = await Promise.all([
        axios.get(`${API}/stats`),
        axios.get(`${API}/builds`)
      ]);
      setStats(statsRes.data);
      setRecentBuilds(buildsRes.data.slice(0, 5));
    } catch (error) {
      console.error('Error fetching dashboard data:', error);
    } finally {
      setLoading(false);
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'completed':
        return <CheckCircle size={20} weight="fill" className="text-[#34C759]" />;
      case 'failed':
        return <XCircle size={20} weight="fill" className="text-[#FF3B30]" />;
      case 'building':
      case 'scanning':
      case 'hardening':
        return <Clock size={20} weight="fill" className="text-[#FFCC00]" />;
      default:
        return <Clock size={20} weight="regular" className="text-[#4B5563]" />;
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-dashboard">
        <div className="text-center">
          <Clock size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Dashboard...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="dashboard-page">
      <div className="mb-8">
        <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>Control Center</h2>
        <p className="text-base text-[#4B5563]">Monitor and manage your hardened container images</p>
      </div>

      {/* Stats Grid */}
      <div className="control-grid mb-12" data-testid="stats-grid">
        <div className="stat-card p-6" data-testid="stat-total-builds">
          <div className="flex items-start justify-between mb-4">
            <div className="bg-[#002FA7]/10 p-3 rounded-sm">
              <Cube size={24} weight="bold" className="text-[#002FA7]" />
            </div>
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Total</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats?.total_builds || 0}</div>
          <div className="text-sm text-[#4B5563] mt-1">Total Builds</div>
        </div>

        <div className="stat-card p-6" data-testid="stat-completed">
          <div className="flex items-start justify-between mb-4">
            <div className="bg-[#34C759]/10 p-3 rounded-sm">
              <CheckCircle size={24} weight="bold" className="text-[#34C759]" />
            </div>
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Success</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats?.completed_builds || 0}</div>
          <div className="text-sm text-[#4B5563] mt-1">Completed</div>
        </div>

        <div className="stat-card p-6" data-testid="stat-in-progress">
          <div className="flex items-start justify-between mb-4">
            <div className="bg-[#FFCC00]/10 p-3 rounded-sm">
              <Clock size={24} weight="bold" className="text-[#FFCC00]" />
            </div>
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Active</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats?.in_progress || 0}</div>
          <div className="text-sm text-[#4B5563] mt-1">In Progress</div>
        </div>

        <div className="stat-card p-6" data-testid="stat-compliance">
          <div className="flex items-start justify-between mb-4">
            <div className="bg-[#002FA7]/10 p-3 rounded-sm">
              <ShieldCheck size={24} weight="bold" className="text-[#002FA7]" />
            </div>
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Score</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats?.avg_compliance_score || 0}%</div>
          <div className="text-sm text-[#4B5563] mt-1">Avg Compliance</div>
        </div>
      </div>

      {/* Recent Builds */}
      <div className="bg-white border border-black/10 rounded-sm p-6" data-testid="recent-builds-section">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-2xl font-bold tracking-tight" style={{fontFamily: 'Chivo'}}>Recent Builds</h3>
          <Link to="/builds" className="text-sm text-[#002FA7] hover:underline" data-testid="view-all-builds-link">View All</Link>
        </div>

        {recentBuilds.length === 0 ? (
          <div className="text-center py-12" data-testid="no-builds-message">
            <Cube size={48} className="mx-auto mb-4 text-[#E5E7EB]" />
            <p className="text-[#4B5563] mb-4">No builds yet</p>
            <Link to="/new" className="btn-primary" data-testid="create-first-build-btn">Create Your First Build</Link>
          </div>
        ) : (
          <div className="space-y-3">
            {recentBuilds.map((build) => (
              <Link
                key={build.id}
                to={`/builds/${build.id}`}
                className="block border border-black/10 rounded-sm p-4 hover:border-black/30 hover:-translate-y-1 hover:shadow-lg transition-all duration-200"
                data-testid={`build-item-${build.id}`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    {getStatusIcon(build.status)}
                    <div>
                      <div className="font-medium">{build.config_name}</div>
                      <div className="text-sm text-[#4B5563]">{build.image_tag || 'Building...'}</div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">{build.status}</div>
                    {build.compliance_score && (
                      <div className="text-sm font-medium">Compliance: {build.compliance_score}%</div>
                    )}
                  </div>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

// New Build Form Component
const NewBuild = () => {
  const navigate = useNavigate();
  const [formData, setFormData] = useState({
    name: '',
    runtime: 'java',
    base_image: 'alpine',
    compliance_profiles: ['cis'],
    architecture: ['amd64'],  // Phase 3: Multi-arch
    remove_shell: true,
    remove_package_manager: true,
    enable_sbom: true,
    enable_signing: true
  });
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setSubmitting(true);

    try {
      const response = await axios.post(`${API}/builds`, formData);
      navigate(`/builds/${response.data.id}`);
    } catch (error) {
      console.error('Error creating build:', error);
      alert('Failed to create build');
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

  return (
    <div className="max-w-4xl mx-auto px-6 sm:px-8 py-8" data-testid="new-build-page">
      <div className="mb-8">
        <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>New Build Configuration</h2>
        <p className="text-base text-[#4B5563]">Configure and start a new hardened image build</p>
      </div>

      <form onSubmit={handleSubmit} className="bg-white border border-black/10 rounded-sm p-8">
        {/* Build Name */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2" htmlFor="name">
            Build Name
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

        {/* Runtime */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Runtime Environment
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

        {/* Base Image */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Base Image
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

        {/* Architecture - Phase 3 */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Target Architecture
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
          <p className="text-xs text-[#4B5563] mt-2">Select one or both architectures for multi-platform builds</p>
        </div>

        {/* Compliance Profiles */}
        <div className="mb-6">
          <label className="block text-sm uppercase tracking-wider font-medium mb-2">
            Compliance Profiles
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

        {/* Hardening Options */}
        <div className="mb-8">
          <label className="block text-sm uppercase tracking-wider font-medium mb-3">
            Hardening Options
          </label>
          <div className="space-y-3">
            <label className="flex items-center gap-3 cursor-pointer" data-testid="option-remove-shell">
              <input
                type="checkbox"
                checked={formData.remove_shell}
                onChange={(e) => setFormData({...formData, remove_shell: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
              />
              <span className="text-sm">Remove shell binaries (sh/bash)</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer" data-testid="option-remove-pkg-mgr">
              <input
                type="checkbox"
                checked={formData.remove_package_manager}
                onChange={(e) => setFormData({...formData, remove_package_manager: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
              />
              <span className="text-sm">Remove package managers (apt/apk)</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer" data-testid="option-enable-sbom">
              <input
                type="checkbox"
                checked={formData.enable_sbom}
                onChange={(e) => setFormData({...formData, enable_sbom: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
              />
              <span className="text-sm">Generate SBOM (CycloneDX)</span>
            </label>
            <label className="flex items-center gap-3 cursor-pointer" data-testid="option-enable-signing">
              <input
                type="checkbox"
                checked={formData.enable_signing}
                onChange={(e) => setFormData({...formData, enable_signing: e.target.checked})}
                className="w-4 h-4 accent-[#002FA7]"
              />
              <span className="text-sm">Sign image with Cosign</span>
            </label>
          </div>
        </div>

        {/* Submit */}
        <div className="flex gap-4">
          <button
            type="submit"
            disabled={submitting}
            className="btn-primary flex-1"
            data-testid="submit-build-btn"
          >
            {submitting ? 'Starting Build...' : 'Start Build'}
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

export default function App() {
  return (
    <div className="App min-h-screen bg-[#F9FAFB]">
      <BrowserRouter>
        <Header />
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/new" element={<EnhancedNewBuild />} />
          <Route path="/builds" element={<BuildsList />} />
          <Route path="/builds/:buildId" element={<BuildDetail />} />
          <Route path="/analytics" element={<Analytics />} />
          <Route path="/policies" element={<Policies />} />
          <Route path="/exceptions" element={<Exceptions />} />
          <Route path="/drift" element={<DriftDetection />} />
          <Route path="/remediation-policies" element={<RemediationPolicies />} />
        </Routes>
      </BrowserRouter>
    </div>
  );
}