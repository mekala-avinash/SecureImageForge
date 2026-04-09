import React, { useState, useEffect } from 'react';
import axios from 'axios';
import {
  ShieldWarning,
  CheckCircle,
  XCircle,
  Clock,
  Plus,
  FileText,
  User,
  CalendarBlank,
  Warning,
  CaretDown,
  CaretUp
} from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const Exceptions = () => {
  const [exceptions, setExceptions] = useState([]);
  const [counts, setCounts] = useState({ pending: 0, approved: 0, rejected: 0, total: 0 });
  const [templates, setTemplates] = useState({});
  const [loading, setLoading] = useState(true);
  const [activeFilter, setActiveFilter] = useState('all');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [expandedExceptions, setExpandedExceptions] = useState({});
  const [newException, setNewException] = useState({
    build_id: '',
    policy_id: '',
    requestor: '',
    justification: '',
    template_type: '',
    duration_days: 30
  });

  useEffect(() => {
    fetchExceptions();
    fetchTemplates();
  }, [activeFilter]);

  const fetchExceptions = async () => {
    try {
      const statusParam = activeFilter !== 'all' ? `?status=${activeFilter}` : '';
      const res = await axios.get(`${API}/exceptions${statusParam}`);
      setExceptions(res.data.exceptions);
      setCounts(res.data.counts);
    } catch (error) {
      console.error('Error fetching exceptions:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchTemplates = async () => {
    try {
      const res = await axios.get(`${API}/exceptions/templates`);
      setTemplates(res.data.templates);
    } catch (error) {
      console.error('Error fetching templates:', error);
    }
  };

  const handleCreateException = async (e) => {
    e.preventDefault();
    try {
      await axios.post(`${API}/exceptions`, newException);
      setShowCreateModal(false);
      setNewException({
        build_id: '',
        policy_id: '',
        requestor: '',
        justification: '',
        template_type: '',
        duration_days: 30
      });
      fetchExceptions();
    } catch (error) {
      alert('Failed to create exception: ' + (error.response?.data?.detail || error.message));
    }
  };

  const handleApprove = async (exceptionId) => {
    const notes = prompt('Approval notes (optional):');
    try {
      await axios.post(`${API}/exceptions/${exceptionId}/approve`, {
        approver: 'current_user',
        notes: notes || ''
      });
      fetchExceptions();
    } catch (error) {
      alert('Failed to approve: ' + (error.response?.data?.detail || error.message));
    }
  };

  const handleReject = async (exceptionId) => {
    const reason = prompt('Rejection reason:');
    if (!reason) return;
    try {
      await axios.post(`${API}/exceptions/${exceptionId}/reject`, {
        approver: 'current_user',
        reason
      });
      fetchExceptions();
    } catch (error) {
      alert('Failed to reject: ' + (error.response?.data?.detail || error.message));
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'approved':
        return <CheckCircle size={20} weight="fill" className="text-[#34C759]" />;
      case 'rejected':
        return <XCircle size={20} weight="fill" className="text-[#FF3B30]" />;
      default:
        return <Clock size={20} weight="fill" className="text-[#FFCC00]" />;
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'approved':
        return 'bg-[#34C759]/10 text-[#34C759] border-[#34C759]/20';
      case 'rejected':
        return 'bg-[#FF3B30]/10 text-[#FF3B30] border-[#FF3B30]/20';
      default:
        return 'bg-[#FFCC00]/10 text-[#FFCC00] border-[#FFCC00]/20';
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-exceptions">
        <div className="text-center">
          <Clock size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Exceptions...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="exceptions-page">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>
            Exception Management
          </h2>
          <p className="text-base text-[#4B5563]">Request and manage policy deviation exceptions</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="btn-primary flex items-center gap-2"
          data-testid="create-exception-btn"
        >
          <Plus size={16} weight="bold" />
          New Exception Request
        </button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4 mb-8" data-testid="exception-stats">
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <Clock size={24} weight="bold" className="text-[#FFCC00]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Pending</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{counts.pending}</div>
        </div>
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <CheckCircle size={24} weight="bold" className="text-[#34C759]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Approved</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{counts.approved}</div>
        </div>
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <XCircle size={24} weight="bold" className="text-[#FF3B30]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Rejected</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{counts.rejected}</div>
        </div>
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <ShieldWarning size={24} weight="bold" className="text-[#002FA7]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Total</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{counts.total}</div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-2 mb-6">
        {['all', 'pending', 'approved', 'rejected'].map(filter => (
          <button
            key={filter}
            onClick={() => setActiveFilter(filter)}
            className={`px-4 py-2 rounded-sm text-sm uppercase tracking-wider font-medium transition-colors ${
              activeFilter === filter
                ? 'bg-[#002FA7] text-white'
                : 'bg-white border border-black/10 text-[#4B5563] hover:border-[#002FA7]'
            }`}
            data-testid={`filter-${filter}`}
          >
            {filter}
          </button>
        ))}
      </div>

      {/* Exception List */}
      <div className="bg-white border border-black/10 rounded-sm" data-testid="exceptions-list">
        {exceptions.length === 0 ? (
          <div className="text-center py-12">
            <ShieldWarning size={48} className="mx-auto mb-4 text-[#E5E7EB]" />
            <p className="text-[#4B5563] mb-4">No exception requests found</p>
            <button
              onClick={() => setShowCreateModal(true)}
              className="btn-primary"
            >
              Create First Exception Request
            </button>
          </div>
        ) : (
          <div className="divide-y divide-black/10">
            {exceptions.map(exception => (
              <div key={exception.id} className="p-4">
                <div 
                  className="flex items-center justify-between cursor-pointer"
                  onClick={() => setExpandedExceptions(prev => ({
                    ...prev, 
                    [exception.id]: !prev[exception.id]
                  }))}
                >
                  <div className="flex items-center gap-4">
                    {getStatusIcon(exception.status)}
                    <div>
                      <div className="font-medium flex items-center gap-2">
                        {exception.template_type ? templates[exception.template_type]?.title : exception.policy_id}
                        <span className={`text-xs px-2 py-0.5 rounded-full border ${getStatusColor(exception.status)}`}>
                          {exception.status.toUpperCase()}
                        </span>
                      </div>
                      <div className="text-sm text-[#4B5563] flex items-center gap-4 mt-1">
                        <span className="flex items-center gap-1">
                          <User size={14} />
                          {exception.requestor}
                        </span>
                        <span className="flex items-center gap-1">
                          <CalendarBlank size={14} />
                          {new Date(exception.created_at).toLocaleDateString()}
                        </span>
                        {exception.duration_days && (
                          <span className="text-xs">{exception.duration_days} day duration</span>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    {exception.status === 'pending' && (
                      <>
                        <button
                          onClick={(e) => { e.stopPropagation(); handleApprove(exception.id); }}
                          className="px-3 py-1 text-sm bg-[#34C759] text-white rounded-sm hover:bg-[#34C759]/90"
                          data-testid={`approve-btn-${exception.id}`}
                        >
                          Approve
                        </button>
                        <button
                          onClick={(e) => { e.stopPropagation(); handleReject(exception.id); }}
                          className="px-3 py-1 text-sm bg-[#FF3B30] text-white rounded-sm hover:bg-[#FF3B30]/90"
                          data-testid={`reject-btn-${exception.id}`}
                        >
                          Reject
                        </button>
                      </>
                    )}
                    {expandedExceptions[exception.id] ? <CaretUp size={16} /> : <CaretDown size={16} />}
                  </div>
                </div>

                {expandedExceptions[exception.id] && (
                  <div className="mt-4 pt-4 border-t border-black/10">
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Build ID</div>
                        <div className="font-mono text-xs">{exception.build_id}</div>
                      </div>
                      <div>
                        <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Policy ID</div>
                        <div className="font-mono text-xs">{exception.policy_id}</div>
                      </div>
                    </div>
                    <div className="mt-3">
                      <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Justification</div>
                      <div className="bg-black/5 p-3 rounded-sm text-sm">{exception.justification}</div>
                    </div>
                    {exception.status === 'approved' && exception.expires_at && (
                      <div className="mt-3 p-3 bg-[#34C759]/10 rounded-sm text-sm">
                        <strong>Approved by:</strong> {exception.approver} | 
                        <strong> Expires:</strong> {new Date(exception.expires_at).toLocaleDateString()}
                      </div>
                    )}
                    {exception.status === 'rejected' && exception.rejection_reason && (
                      <div className="mt-3 p-3 bg-[#FF3B30]/10 rounded-sm text-sm">
                        <strong>Rejected by:</strong> {exception.approver} | 
                        <strong> Reason:</strong> {exception.rejection_reason}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Exception Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" data-testid="create-exception-modal">
          <div className="bg-white rounded-sm p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto">
            <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>New Exception Request</h3>
            
            <form onSubmit={handleCreateException}>
              {/* Template Selection */}
              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Template (Optional)</label>
                <select
                  value={newException.template_type}
                  onChange={(e) => {
                    const template = templates[e.target.value];
                    setNewException({
                      ...newException,
                      template_type: e.target.value,
                      policy_id: e.target.value,
                      duration_days: template?.default_duration_days || 30
                    });
                  }}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  data-testid="select-template"
                >
                  <option value="">Select a template...</option>
                  {Object.entries(templates).map(([key, template]) => (
                    <option key={key} value={key}>{template.title}</option>
                  ))}
                </select>
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Build ID *</label>
                <input
                  type="text"
                  value={newException.build_id}
                  onChange={(e) => setNewException({...newException, build_id: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono"
                  placeholder="Build ID or 'global' for policy-wide exception"
                  required
                  data-testid="input-build-id"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Policy ID *</label>
                <input
                  type="text"
                  value={newException.policy_id}
                  onChange={(e) => setNewException({...newException, policy_id: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono"
                  placeholder="Policy to request exception for"
                  required
                  data-testid="input-policy-id"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Requestor *</label>
                <input
                  type="text"
                  value={newException.requestor}
                  onChange={(e) => setNewException({...newException, requestor: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  placeholder="Your name or email"
                  required
                  data-testid="input-requestor"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Duration (Days)</label>
                <input
                  type="number"
                  value={newException.duration_days}
                  onChange={(e) => setNewException({...newException, duration_days: parseInt(e.target.value)})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  min="1"
                  max="365"
                  data-testid="input-duration"
                />
              </div>

              <div className="mb-6">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Justification *</label>
                <textarea
                  value={newException.justification}
                  onChange={(e) => setNewException({...newException, justification: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm h-24"
                  placeholder="Explain why this exception is needed and what mitigation is in place..."
                  required
                  data-testid="input-justification"
                />
              </div>

              <div className="flex gap-3">
                <button type="submit" className="btn-primary flex-1" data-testid="submit-exception-btn">
                  Submit Request
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
