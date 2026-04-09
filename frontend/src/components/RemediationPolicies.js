import React, { useState, useEffect } from 'react';
import axios from 'axios';
import {
  Gear,
  CheckCircle,
  Warning,
  ShieldCheck,
  Lightning,
  Bell,
  XCircle,
  Plus,
  RadioButton
} from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const RemediationPolicies = () => {
  const [policies, setPolicies] = useState([]);
  const [activePolicy, setActivePolicy] = useState(null);
  const [stats, setStats] = useState(null);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newPolicy, setNewPolicy] = useState({
    name: '',
    description: '',
    mode: 'graceful',
    auto_remediate_critical: true,
    auto_remediate_high: true,
    auto_remediate_medium: false,
    fail_on_unfixable_critical: false,
    notify_on_remediation: true
  });

  useEffect(() => {
    fetchPolicies();
    fetchStats();
  }, []);

  const fetchPolicies = async () => {
    try {
      const res = await axios.get(`${API}/remediation/policies`);
      setPolicies(res.data.policies);
      setActivePolicy(res.data.active_policy);
    } catch (error) {
      console.error('Error fetching policies:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchStats = async () => {
    try {
      const res = await axios.get(`${API}/remediation/stats`);
      setStats(res.data);
    } catch (error) {
      console.error('Error fetching stats:', error);
    }
  };

  const handleActivatePolicy = async (policyId) => {
    try {
      await axios.post(`${API}/remediation/policies/${policyId}/activate`);
      fetchPolicies();
    } catch (error) {
      alert('Failed to activate policy: ' + (error.response?.data?.detail || error.message));
    }
  };

  const handleCreatePolicy = async (e) => {
    e.preventDefault();
    try {
      await axios.post(`${API}/remediation/policies`, newPolicy);
      setShowCreateModal(false);
      setNewPolicy({
        name: '',
        description: '',
        mode: 'graceful',
        auto_remediate_critical: true,
        auto_remediate_high: true,
        auto_remediate_medium: false,
        fail_on_unfixable_critical: false,
        notify_on_remediation: true
      });
      fetchPolicies();
    } catch (error) {
      alert('Failed to create policy: ' + (error.response?.data?.detail || error.message));
    }
  };

  const getModeIcon = (mode) => {
    switch (mode) {
      case 'strict':
        return <ShieldCheck size={20} weight="fill" className="text-[#FF3B30]" />;
      case 'graceful':
        return <Lightning size={20} weight="fill" className="text-[#34C759]" />;
      default:
        return <Bell size={20} weight="fill" className="text-[#FFCC00]" />;
    }
  };

  const getModeColor = (mode) => {
    switch (mode) {
      case 'strict':
        return 'border-[#FF3B30]/30 bg-[#FF3B30]/5';
      case 'graceful':
        return 'border-[#34C759]/30 bg-[#34C759]/5';
      default:
        return 'border-[#FFCC00]/30 bg-[#FFCC00]/5';
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-policies">
        <div className="text-center">
          <Gear size={48} className="mx-auto mb-4 animate-spin text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Policies...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="remediation-policies-page">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>
            Remediation Policies
          </h2>
          <p className="text-base text-[#4B5563]">Configure automatic vulnerability remediation behavior</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="btn-primary flex items-center gap-2"
          data-testid="create-policy-btn"
        >
          <Plus size={16} weight="bold" />
          Create Policy
        </button>
      </div>

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-4 gap-4 mb-8" data-testid="remediation-stats">
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <Lightning size={24} weight="bold" className="text-[#34C759]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Total Remediations</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.total_remediations_performed}</div>
          </div>
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <CheckCircle size={24} weight="bold" className="text-[#002FA7]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Fixes Applied</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.total_fixes_applied}</div>
          </div>
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <Warning size={24} weight="bold" className="text-[#FFCC00]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">CVE Database</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.cve_database_size}</div>
          </div>
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <ShieldCheck size={24} weight="bold" className="text-[#34C759]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Auto-Fixable</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.auto_fixable_cves}</div>
          </div>
        </div>
      )}

      {/* Active Policy Banner */}
      {activePolicy && (
        <div className="mb-6 p-4 bg-[#002FA7]/10 border border-[#002FA7]/30 rounded-sm" data-testid="active-policy-banner">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              {getModeIcon(activePolicy.mode)}
              <div>
                <div className="font-bold text-[#002FA7]">Active Policy: {activePolicy.name}</div>
                <div className="text-sm text-[#4B5563]">{activePolicy.description}</div>
              </div>
            </div>
            <span className="px-3 py-1 bg-[#002FA7] text-white rounded-full text-xs uppercase tracking-wider">
              {activePolicy.mode} Mode
            </span>
          </div>
        </div>
      )}

      {/* Policies List */}
      <div className="bg-white border border-black/10 rounded-sm" data-testid="policies-list">
        <div className="p-4 border-b border-black/10">
          <h3 className="text-lg font-bold" style={{fontFamily: 'Chivo'}}>Available Policies</h3>
        </div>
        
        <div className="divide-y divide-black/10">
          {policies.map(policy => (
            <div 
              key={policy.id} 
              className={`p-4 ${policy.enabled ? getModeColor(policy.mode) : ''}`}
              data-testid={`policy-${policy.id}`}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3">
                  {getModeIcon(policy.mode)}
                  <div>
                    <div className="font-medium flex items-center gap-2">
                      {policy.name}
                      {policy.enabled && (
                        <span className="text-xs px-2 py-0.5 bg-[#002FA7] text-white rounded-full">ACTIVE</span>
                      )}
                    </div>
                    <div className="text-sm text-[#4B5563] mt-1">{policy.description}</div>
                    
                    {/* Policy Settings */}
                    <div className="mt-3 flex flex-wrap gap-2">
                      <span className={`text-xs px-2 py-1 rounded-sm ${policy.auto_remediate_critical ? 'bg-[#FF3B30]/10 text-[#FF3B30]' : 'bg-black/5 text-[#4B5563]'}`}>
                        Critical: {policy.auto_remediate_critical ? 'Auto-fix' : 'Skip'}
                      </span>
                      <span className={`text-xs px-2 py-1 rounded-sm ${policy.auto_remediate_high ? 'bg-[#FFCC00]/10 text-[#FFCC00]' : 'bg-black/5 text-[#4B5563]'}`}>
                        High: {policy.auto_remediate_high ? 'Auto-fix' : 'Skip'}
                      </span>
                      <span className={`text-xs px-2 py-1 rounded-sm ${policy.auto_remediate_medium ? 'bg-[#002FA7]/10 text-[#002FA7]' : 'bg-black/5 text-[#4B5563]'}`}>
                        Medium: {policy.auto_remediate_medium ? 'Auto-fix' : 'Skip'}
                      </span>
                      {policy.fail_on_unfixable_critical && (
                        <span className="text-xs px-2 py-1 rounded-sm bg-[#FF3B30]/10 text-[#FF3B30]">
                          Fail on unfixable critical
                        </span>
                      )}
                      {policy.notify_on_remediation && (
                        <span className="text-xs px-2 py-1 rounded-sm bg-[#34C759]/10 text-[#34C759]">
                          Notifications enabled
                        </span>
                      )}
                    </div>
                  </div>
                </div>
                
                {!policy.enabled && (
                  <button
                    onClick={() => handleActivatePolicy(policy.id)}
                    className="flex items-center gap-2 px-3 py-1 border border-[#002FA7] text-[#002FA7] rounded-sm text-sm hover:bg-[#002FA7]/10"
                    data-testid={`activate-btn-${policy.id}`}
                  >
                    <RadioButton size={14} />
                    Activate
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Mode Comparison */}
      <div className="mt-8 bg-white border border-black/10 rounded-sm p-6" data-testid="mode-comparison">
        <h3 className="text-lg font-bold mb-4" style={{fontFamily: 'Chivo'}}>Policy Mode Comparison</h3>
        <div className="grid grid-cols-3 gap-4">
          <div className="p-4 border border-[#FF3B30]/30 rounded-sm bg-[#FF3B30]/5">
            <div className="flex items-center gap-2 mb-2">
              <ShieldCheck size={20} weight="fill" className="text-[#FF3B30]" />
              <h4 className="font-bold">Strict Mode</h4>
            </div>
            <ul className="text-sm text-[#4B5563] space-y-1">
              <li>• Blocks builds with unfixable critical CVEs</li>
              <li>• Auto-fixes all remediate-able vulnerabilities</li>
              <li>• Enforces security compliance</li>
              <li>• Best for: Production environments</li>
            </ul>
          </div>
          <div className="p-4 border border-[#34C759]/30 rounded-sm bg-[#34C759]/5">
            <div className="flex items-center gap-2 mb-2">
              <Lightning size={20} weight="fill" className="text-[#34C759]" />
              <h4 className="font-bold">Graceful Mode</h4>
            </div>
            <ul className="text-sm text-[#4B5563] space-y-1">
              <li>• Applies available fixes automatically</li>
              <li>• Allows builds even with remaining CVEs</li>
              <li>• Balances security with velocity</li>
              <li>• Best for: Development/Staging</li>
            </ul>
          </div>
          <div className="p-4 border border-[#FFCC00]/30 rounded-sm bg-[#FFCC00]/5">
            <div className="flex items-center gap-2 mb-2">
              <Bell size={20} weight="fill" className="text-[#FFCC00]" />
              <h4 className="font-bold">Notify Only</h4>
            </div>
            <ul className="text-sm text-[#4B5563] space-y-1">
              <li>• Detects but doesn't auto-fix</li>
              <li>• Sends notifications for review</li>
              <li>• Manual remediation required</li>
              <li>• Best for: Compliance auditing</li>
            </ul>
          </div>
        </div>
      </div>

      {/* Create Policy Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" data-testid="create-policy-modal">
          <div className="bg-white rounded-sm p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto">
            <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Create Remediation Policy</h3>
            
            <form onSubmit={handleCreatePolicy}>
              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Policy Name *</label>
                <input
                  type="text"
                  value={newPolicy.name}
                  onChange={(e) => setNewPolicy({...newPolicy, name: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  placeholder="My Custom Policy"
                  required
                  data-testid="input-policy-name"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Description</label>
                <textarea
                  value={newPolicy.description}
                  onChange={(e) => setNewPolicy({...newPolicy, description: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm h-20"
                  placeholder="Describe this policy's purpose..."
                  data-testid="input-policy-description"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Mode *</label>
                <select
                  value={newPolicy.mode}
                  onChange={(e) => setNewPolicy({...newPolicy, mode: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  data-testid="select-mode"
                >
                  <option value="strict">Strict - Block on unfixable critical CVEs</option>
                  <option value="graceful">Graceful - Fix what we can, allow rest</option>
                  <option value="notify_only">Notify Only - No auto-fix</option>
                </select>
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Auto-Remediation Settings</label>
                <div className="space-y-2">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={newPolicy.auto_remediate_critical}
                      onChange={(e) => setNewPolicy({...newPolicy, auto_remediate_critical: e.target.checked})}
                      className="w-4 h-4 accent-[#002FA7]"
                    />
                    <span className="text-sm">Auto-remediate Critical vulnerabilities</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={newPolicy.auto_remediate_high}
                      onChange={(e) => setNewPolicy({...newPolicy, auto_remediate_high: e.target.checked})}
                      className="w-4 h-4 accent-[#002FA7]"
                    />
                    <span className="text-sm">Auto-remediate High vulnerabilities</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={newPolicy.auto_remediate_medium}
                      onChange={(e) => setNewPolicy({...newPolicy, auto_remediate_medium: e.target.checked})}
                      className="w-4 h-4 accent-[#002FA7]"
                    />
                    <span className="text-sm">Auto-remediate Medium vulnerabilities</span>
                  </label>
                </div>
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Additional Settings</label>
                <div className="space-y-2">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={newPolicy.fail_on_unfixable_critical}
                      onChange={(e) => setNewPolicy({...newPolicy, fail_on_unfixable_critical: e.target.checked})}
                      className="w-4 h-4 accent-[#002FA7]"
                    />
                    <span className="text-sm">Fail build if unfixable critical CVEs remain</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={newPolicy.notify_on_remediation}
                      onChange={(e) => setNewPolicy({...newPolicy, notify_on_remediation: e.target.checked})}
                      className="w-4 h-4 accent-[#002FA7]"
                    />
                    <span className="text-sm">Send notifications on remediation</span>
                  </label>
                </div>
              </div>

              <div className="flex gap-3">
                <button type="submit" className="btn-primary flex-1" data-testid="submit-policy-btn">
                  Create Policy
                </button>
                <button 
                  type="button" 
                  onClick={() => setShowCreateModal(false)}
                  className="btn-secondary"
                >
                  Cancel
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
