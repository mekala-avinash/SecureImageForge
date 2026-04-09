import React, { useState, useEffect } from 'react';
import axios from 'axios';
import {
  Link,
  Plus,
  Trash,
  CheckCircle,
  XCircle,
  Clock,
  PaperPlaneTilt,
  SlackLogo,
  Globe,
  CaretDown,
  CaretUp
} from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const Webhooks = () => {
  const [webhooks, setWebhooks] = useState([]);
  const [stats, setStats] = useState(null);
  const [events, setEvents] = useState([]);
  const [destinations, setDestinations] = useState([]);
  const [deliveryHistory, setDeliveryHistory] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [expandedWebhooks, setExpandedWebhooks] = useState({});
  const [newWebhook, setNewWebhook] = useState({
    name: '',
    destination: 'slack',
    url: '',
    events: [],
    channel: '',
    secret: '',
    enabled: true
  });

  useEffect(() => {
    fetchData();
  }, []);

  const fetchData = async () => {
    try {
      const [webhooksRes, eventsRes, destinationsRes, historyRes] = await Promise.all([
        axios.get(`${API}/webhooks`),
        axios.get(`${API}/webhooks/events`),
        axios.get(`${API}/webhooks/destinations`),
        axios.get(`${API}/webhooks/delivery-history?limit=20`)
      ]);
      
      setWebhooks(webhooksRes.data.webhooks);
      setStats(webhooksRes.data.stats);
      setEvents(eventsRes.data.events);
      setDestinations(destinationsRes.data.destinations);
      setDeliveryHistory(historyRes.data.deliveries);
    } catch (error) {
      console.error('Error fetching webhooks:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateWebhook = async (e) => {
    e.preventDefault();
    try {
      await axios.post(`${API}/webhooks`, newWebhook);
      setShowCreateModal(false);
      setNewWebhook({
        name: '',
        destination: 'slack',
        url: '',
        events: [],
        channel: '',
        secret: '',
        enabled: true
      });
      fetchData();
    } catch (error) {
      alert('Failed to create webhook: ' + (error.response?.data?.detail || error.message));
    }
  };

  const handleDeleteWebhook = async (webhookId) => {
    if (!window.confirm('Are you sure you want to delete this webhook?')) return;
    try {
      await axios.delete(`${API}/webhooks/${webhookId}`);
      fetchData();
    } catch (error) {
      alert('Failed to delete webhook: ' + (error.response?.data?.detail || error.message));
    }
  };

  const handleTestWebhook = async (webhookId) => {
    try {
      const res = await axios.post(`${API}/webhooks/${webhookId}/test`);
      alert(res.data.status === 'success' ? 'Test webhook sent successfully!' : 'Test failed: ' + res.data.message);
      fetchData();
    } catch (error) {
      alert('Test failed: ' + (error.response?.data?.detail || error.message));
    }
  };

  const getDestinationIcon = (destination) => {
    switch (destination) {
      case 'slack':
        return <SlackLogo size={20} weight="fill" className="text-[#4A154B]" />;
      case 'microsoft_teams':
        return <Globe size={20} className="text-[#6264A7]" />;
      case 'discord':
        return <Globe size={20} className="text-[#5865F2]" />;
      default:
        return <Globe size={20} className="text-[#4B5563]" />;
    }
  };

  const toggleEventSelection = (eventId) => {
    if (newWebhook.events.includes(eventId)) {
      setNewWebhook({
        ...newWebhook,
        events: newWebhook.events.filter(e => e !== eventId)
      });
    } else {
      setNewWebhook({
        ...newWebhook,
        events: [...newWebhook.events, eventId]
      });
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-webhooks">
        <div className="text-center">
          <Clock size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Webhooks...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="webhooks-page">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>
            Webhook Integrations
          </h2>
          <p className="text-base text-[#4B5563]">Configure ChatOps notifications for Slack, Teams, Discord</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="btn-primary flex items-center gap-2"
          data-testid="create-webhook-btn"
        >
          <Plus size={16} weight="bold" />
          Add Webhook
        </button>
      </div>

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-4 gap-4 mb-8" data-testid="webhook-stats">
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <Link size={24} weight="bold" className="text-[#002FA7]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Registered</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.registered_webhooks}</div>
          </div>
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <CheckCircle size={24} weight="bold" className="text-[#34C759]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Enabled</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.enabled_webhooks}</div>
          </div>
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <PaperPlaneTilt size={24} weight="bold" className="text-[#002FA7]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Deliveries</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.total_deliveries}</div>
          </div>
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <CheckCircle size={24} weight="bold" className="text-[#34C759]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Success Rate</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats.success_rate}%</div>
          </div>
        </div>
      )}

      <div className="grid grid-cols-2 gap-6">
        {/* Webhooks List */}
        <div className="bg-white border border-black/10 rounded-sm" data-testid="webhooks-list">
          <div className="p-4 border-b border-black/10">
            <h3 className="text-lg font-bold" style={{fontFamily: 'Chivo'}}>Registered Webhooks</h3>
          </div>
          
          {webhooks.length === 0 ? (
            <div className="text-center py-12">
              <Link size={48} className="mx-auto mb-4 text-[#E5E7EB]" />
              <p className="text-[#4B5563] mb-4">No webhooks configured</p>
              <button onClick={() => setShowCreateModal(true)} className="btn-primary">
                Add Your First Webhook
              </button>
            </div>
          ) : (
            <div className="divide-y divide-black/10">
              {webhooks.map(webhook => (
                <div key={webhook.id} className="p-4">
                  <div 
                    className="flex items-center justify-between cursor-pointer"
                    onClick={() => setExpandedWebhooks(prev => ({...prev, [webhook.id]: !prev[webhook.id]}))}
                  >
                    <div className="flex items-center gap-3">
                      {getDestinationIcon(webhook.destination)}
                      <div>
                        <div className="font-medium flex items-center gap-2">
                          {webhook.name}
                          {webhook.enabled ? (
                            <span className="text-xs px-2 py-0.5 bg-[#34C759]/10 text-[#34C759] rounded-full">ACTIVE</span>
                          ) : (
                            <span className="text-xs px-2 py-0.5 bg-black/10 text-[#4B5563] rounded-full">DISABLED</span>
                          )}
                        </div>
                        <div className="text-xs text-[#4B5563]">{webhook.url}</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={(e) => { e.stopPropagation(); handleTestWebhook(webhook.id); }}
                        className="px-2 py-1 text-xs border border-black/20 rounded-sm hover:border-[#002FA7]"
                        data-testid={`test-webhook-${webhook.id}`}
                      >
                        Test
                      </button>
                      <button
                        onClick={(e) => { e.stopPropagation(); handleDeleteWebhook(webhook.id); }}
                        className="px-2 py-1 text-xs text-[#FF3B30] border border-[#FF3B30]/30 rounded-sm hover:bg-[#FF3B30]/10"
                        data-testid={`delete-webhook-${webhook.id}`}
                      >
                        <Trash size={14} />
                      </button>
                      {expandedWebhooks[webhook.id] ? <CaretUp size={16} /> : <CaretDown size={16} />}
                    </div>
                  </div>
                  
                  {expandedWebhooks[webhook.id] && (
                    <div className="mt-3 pt-3 border-t border-black/10">
                      <div className="text-xs text-[#4B5563] mb-2">Subscribed Events:</div>
                      <div className="flex flex-wrap gap-1">
                        {webhook.events.map(event => (
                          <span key={event} className="text-xs px-2 py-1 bg-black/5 rounded-sm">
                            {event}
                          </span>
                        ))}
                      </div>
                      <div className="mt-2 text-xs text-[#4B5563]">
                        Deliveries: {webhook.delivery_count} | Failures: {webhook.failure_count}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Delivery History */}
        <div className="bg-white border border-black/10 rounded-sm" data-testid="delivery-history">
          <div className="p-4 border-b border-black/10">
            <h3 className="text-lg font-bold" style={{fontFamily: 'Chivo'}}>Recent Deliveries</h3>
          </div>
          
          {deliveryHistory.length === 0 ? (
            <div className="text-center py-12">
              <PaperPlaneTilt size={48} className="mx-auto mb-4 text-[#E5E7EB]" />
              <p className="text-[#4B5563]">No deliveries yet</p>
            </div>
          ) : (
            <div className="divide-y divide-black/10 max-h-[500px] overflow-y-auto">
              {deliveryHistory.map((delivery, idx) => (
                <div key={idx} className="p-3 flex items-center gap-3">
                  {delivery.status === 'success' ? (
                    <CheckCircle size={16} weight="fill" className="text-[#34C759]" />
                  ) : delivery.status === 'failed' ? (
                    <XCircle size={16} weight="fill" className="text-[#FF3B30]" />
                  ) : (
                    <Clock size={16} className="text-[#FFCC00]" />
                  )}
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">{delivery.event_type}</div>
                    <div className="text-xs text-[#4B5563]">{delivery.webhook_name}</div>
                  </div>
                  <div className="text-xs text-[#4B5563]">
                    {delivery.response_code && `${delivery.response_code}`}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Create Webhook Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" data-testid="create-webhook-modal">
          <div className="bg-white rounded-sm p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto">
            <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Add Webhook</h3>
            
            <form onSubmit={handleCreateWebhook}>
              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Name *</label>
                <input
                  type="text"
                  value={newWebhook.name}
                  onChange={(e) => setNewWebhook({...newWebhook, name: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  placeholder="Production Alerts"
                  required
                  data-testid="input-webhook-name"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Destination *</label>
                <select
                  value={newWebhook.destination}
                  onChange={(e) => setNewWebhook({...newWebhook, destination: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  data-testid="select-destination"
                >
                  {destinations.map(dest => (
                    <option key={dest.id} value={dest.id}>{dest.name}</option>
                  ))}
                </select>
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Webhook URL *</label>
                <input
                  type="url"
                  value={newWebhook.url}
                  onChange={(e) => setNewWebhook({...newWebhook, url: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm font-mono"
                  placeholder="https://hooks.slack.com/services/..."
                  required
                  data-testid="input-webhook-url"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Events *</label>
                <div className="border border-black/20 rounded-sm max-h-40 overflow-y-auto">
                  {events.map(event => (
                    <label 
                      key={event.id} 
                      className="flex items-center gap-2 px-3 py-2 hover:bg-black/5 cursor-pointer"
                    >
                      <input
                        type="checkbox"
                        checked={newWebhook.events.includes(event.id)}
                        onChange={() => toggleEventSelection(event.id)}
                        className="w-4 h-4 accent-[#002FA7]"
                      />
                      <span className="text-sm">{event.name}</span>
                    </label>
                  ))}
                </div>
                <div className="text-xs text-[#4B5563] mt-1">
                  {newWebhook.events.length} events selected
                </div>
              </div>

              <div className="mb-4">
                <label className="block text-sm uppercase tracking-wider font-medium mb-2">Channel (Optional)</label>
                <input
                  type="text"
                  value={newWebhook.channel}
                  onChange={(e) => setNewWebhook({...newWebhook, channel: e.target.value})}
                  className="w-full border border-black/20 rounded-sm px-3 py-2 text-sm"
                  placeholder="#security-alerts"
                  data-testid="input-channel"
                />
              </div>

              <div className="flex gap-3">
                <button 
                  type="submit" 
                  className="btn-primary flex-1"
                  disabled={newWebhook.events.length === 0}
                  data-testid="submit-webhook-btn"
                >
                  Create Webhook
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
