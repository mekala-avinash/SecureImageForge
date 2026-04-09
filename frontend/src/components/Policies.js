import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { ShieldCheck, Plus, Trash, ToggleLeft, ToggleRight } from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const Policies = () => {
  const [policies, setPolicies] = useState([]);
  const [templates, setTemplates] = useState({});
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState(null);

  useEffect(() => {
    fetchPolicies();
    fetchTemplates();
  }, []);

  const fetchPolicies = async () => {
    try {
      const response = await axios.get(`${API}/policies`);
      setPolicies(response.data);
    } catch (error) {
      console.error('Error fetching policies:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchTemplates = async () => {
    try {
      const response = await axios.get(`${API}/policies/templates`);
      setTemplates(response.data.templates);
    } catch (error) {
      console.error('Error fetching templates:', error);
    }
  };

  const addPolicyFromTemplate = async (templateKey) => {
    const template = templates[templateKey];
    try {
      await axios.post(`${API}/policies`, template);
      fetchPolicies();
      setShowAddModal(false);
    } catch (error) {
      console.error('Error adding policy:', error);
      alert('Failed to add policy');
    }
  };

  const togglePolicy = async (policyId) => {
    try {
      await axios.post(`${API}/policies/${policyId}/toggle`);
      fetchPolicies();
    } catch (error) {
      console.error('Error toggling policy:', error);
    }
  };

  const deletePolicy = async (policyId) => {
    if (!window.confirm('Are you sure you want to delete this policy?')) return;
    
    try {
      await axios.delete(`${API}/policies/${policyId}`);
      fetchPolicies();
    } catch (error) {
      console.error('Error deleting policy:', error);
    }
  };

  const getEnforcementColor = (enforcement) => {
    const colors = {
      block: 'bg-[#FF3B30]/10 text-[#FF3B30] border-[#FF3B30]/20',
      warn: 'bg-[#FFCC00]/10 text-[#FFCC00] border-[#FFCC00]/20',
      info: 'bg-[#002FA7]/10 text-[#002FA7] border-[#002FA7]/20'
    };
    return colors[enforcement] || colors.info;
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <ShieldCheck size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Policies...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="policies-page">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>Security Policies</h2>
          <p className="text-base text-[#4B5563]">Define and enforce custom security policies</p>
        </div>
        <button
          onClick={() => setShowAddModal(true)}
          className="btn-primary flex items-center gap-2"
          data-testid="add-policy-btn"
        >
          <Plus size={16} weight="bold" />
          Add Policy
        </button>
      </div>

      {/* Active Policies */}
      {policies.length > 0 ? (
        <div className="space-y-3" data-testid="policies-list">
          {policies.map((policy) => (
            <div
              key={policy.id}
              className="bg-white border border-black/10 rounded-sm p-4"
              data-testid={`policy-${policy.id}`}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <h3 className="text-lg font-bold" style={{fontFamily: 'Chivo'}}>{policy.name}</h3>
                    <span className={`text-xs px-2 py-1 rounded-sm border ${getEnforcementColor(policy.enforcement)}`}>
                      {policy.enforcement.toUpperCase()}
                    </span>
                    <span className="text-xs px-2 py-1 bg-black/5 rounded-sm uppercase tracking-wider">
                      {policy.type}
                    </span>
                  </div>
                  <p className="text-sm text-[#4B5563] mb-2">{policy.description}</p>
                  <div className="text-xs font-mono text-[#4B5563]">
                    Rule: {policy.rule.condition} {policy.rule.operator} {JSON.stringify(policy.rule.value)}
                  </div>
                </div>
                
                <div className="flex items-center gap-2 ml-4">
                  <button
                    onClick={() => togglePolicy(policy.id)}
                    className={`p-2 rounded-sm transition-colors ${
                      policy.enabled
                        ? 'bg-[#34C759]/10 text-[#34C759] hover:bg-[#34C759]/20'
                        : 'bg-black/5 text-[#4B5563] hover:bg-black/10'
                    }`}
                    data-testid={`toggle-policy-${policy.id}`}
                    title={policy.enabled ? 'Disable policy' : 'Enable policy'}
                  >
                    {policy.enabled ? <ToggleRight size={24} weight="fill" /> : <ToggleLeft size={24} weight="fill" />}
                  </button>
                  <button
                    onClick={() => deletePolicy(policy.id)}
                    className="p-2 rounded-sm bg-[#FF3B30]/10 text-[#FF3B30] hover:bg-[#FF3B30]/20 transition-colors"
                    data-testid={`delete-policy-${policy.id}`}
                    title="Delete policy"
                  >
                    <Trash size={20} />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="bg-white border border-black/10 rounded-sm p-12 text-center">
          <ShieldCheck size={64} className="mx-auto mb-4 text-[#E5E7EB]" />
          <p className="text-[#4B5563] mb-4">No policies configured</p>
          <button onClick={() => setShowAddModal(true)} className="btn-primary">
            Add Your First Policy
          </button>
        </div>
      )}

      {/* Add Policy Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowAddModal(false)}>
          <div className="bg-white rounded-sm p-6 max-w-4xl w-full mx-4 max-h-[80vh] overflow-y-auto" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-2xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Add Policy from Template</h3>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {Object.entries(templates).map(([key, template]) => (
                <div
                  key={key}
                  className="border border-black/10 rounded-sm p-4 hover:border-[#002FA7] hover:bg-[#002FA7]/5 transition-all cursor-pointer"
                  onClick={() => addPolicyFromTemplate(key)}
                >
                  <div className="flex items-start justify-between mb-2">
                    <h4 className="font-bold" style={{fontFamily: 'Chivo'}}>{template.name}</h4>
                    <span className={`text-xs px-2 py-1 rounded-sm border ${getEnforcementColor(template.enforcement)}`}>
                      {template.enforcement.toUpperCase()}
                    </span>
                  </div>
                  <p className="text-sm text-[#4B5563] mb-2">{template.description}</p>
                  <div className="text-xs uppercase tracking-wider text-[#4B5563]">Type: {template.type}</div>
                </div>
              ))}
            </div>

            <div className="mt-6 flex justify-end">
              <button onClick={() => setShowAddModal(false)} className="btn-secondary">
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
