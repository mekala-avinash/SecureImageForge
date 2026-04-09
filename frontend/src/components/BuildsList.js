import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import axios from 'axios';
import {
  CheckCircle,
  XCircle,
  Clock,
  Cube,
  MagnifyingGlass
} from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const BuildsList = () => {
  const [builds, setBuilds] = useState([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('all');
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    fetchBuilds();
    const interval = setInterval(fetchBuilds, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchBuilds = async () => {
    try {
      const response = await axios.get(`${API}/builds`);
      setBuilds(response.data);
    } catch (error) {
      console.error('Error fetching builds:', error);
    } finally {
      setLoading(false);
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'completed':
        return <CheckCircle size={24} weight="fill" className="text-[#34C759]" />;
      case 'failed':
        return <XCircle size={24} weight="fill" className="text-[#FF3B30]" />;
      case 'building':
      case 'scanning':
      case 'hardening':
        return <Clock size={24} weight="fill" className="text-[#FFCC00]" />;
      default:
        return <Clock size={24} weight="regular" className="text-[#4B5563]" />;
    }
  };

  const filteredBuilds = builds.filter(build => {
    const matchesFilter = filter === 'all' || build.status === filter;
    const matchesSearch = build.config_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                          (build.image_tag && build.image_tag.toLowerCase().includes(searchTerm.toLowerCase()));
    return matchesFilter && matchesSearch;
  });

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-builds-list">
        <div className="text-center">
          <Clock size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Builds...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="builds-list-page">
      <div className="mb-8">
        <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>Build History</h2>
        <p className="text-base text-[#4B5563]">View and manage all image builds</p>
      </div>

      {/* Filters */}
      <div className="bg-white border border-black/10 rounded-sm p-4 mb-6" data-testid="builds-filters">
        <div className="flex flex-col md:flex-row gap-4">
          {/* Search */}
          <div className="flex-1 relative">
            <MagnifyingGlass size={20} className="absolute left-3 top-1/2 -translate-y-1/2 text-[#4B5563]" />
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search builds..."
              className="w-full pl-10 pr-4 py-2 border border-black/20 rounded-sm focus:ring-2 focus:ring-[#002FA7]/30 focus:border-[#002FA7] outline-none transition-all bg-white font-mono text-sm"
              data-testid="search-builds-input"
            />
          </div>

          {/* Status Filter */}
          <div className="flex gap-2">
            {['all', 'completed', 'failed', 'building'].map((status) => (
              <button
                key={status}
                onClick={() => setFilter(status)}
                className={`px-4 py-2 text-xs uppercase tracking-wider font-medium rounded-sm transition-all ${
                  filter === status
                    ? 'bg-[#002FA7] text-white'
                    : 'bg-black/5 text-[#4B5563] hover:bg-black/10'
                }`}
                data-testid={`filter-${status}`}
              >
                {status}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Builds List */}
      {filteredBuilds.length === 0 ? (
        <div className="bg-white border border-black/10 rounded-sm p-12 text-center" data-testid="no-builds-found">
          <Cube size={64} className="mx-auto mb-4 text-[#E5E7EB]" />
          <p className="text-[#4B5563] mb-4">No builds found</p>
          <Link to="/new" className="btn-primary" data-testid="create-build-btn">Create New Build</Link>
        </div>
      ) : (
        <div className="space-y-3" data-testid="builds-list">
          {filteredBuilds.map((build) => (
            <Link
              key={build.id}
              to={`/builds/${build.id}`}
              className="block bg-white border border-black/10 rounded-sm p-4 hover:border-black/30 hover:-translate-y-1 hover:shadow-lg transition-all duration-200"
              data-testid={`build-list-item-${build.id}`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4 flex-1">
                  {getStatusIcon(build.status)}
                  <div className="flex-1">
                    <div className="font-bold text-lg" style={{fontFamily: 'Chivo'}}>{build.config_name}</div>
                    <div className="text-sm text-[#4B5563] font-mono">{build.image_tag || 'Building...'}</div>
                  </div>
                </div>

                <div className="flex items-center gap-8">
                  {build.compliance_score && (
                    <div className="text-center">
                      <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Compliance</div>
                      <div className="text-xl font-bold" style={{fontFamily: 'Chivo'}}>{build.compliance_score}%</div>
                    </div>
                  )}

                  {build.vulnerability_count && (
                    <div className="text-center">
                      <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Vulnerabilities</div>
                      <div className="text-sm font-mono">
                        <span className="text-[#FF3B30] font-bold">C:{build.vulnerability_count.CRITICAL || 0}</span>
                        {' / '}
                        <span className="text-[#FFCC00] font-bold">H:{build.vulnerability_count.HIGH || 0}</span>
                      </div>
                    </div>
                  )}

                  <div className="text-center min-w-[100px]">
                    <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Status</div>
                    <div className={`text-sm font-bold uppercase ${
                      build.status === 'completed' ? 'text-[#34C759]' :
                      build.status === 'failed' ? 'text-[#FF3B30]' :
                      'text-[#FFCC00]'
                    }`}>
                      {build.status}
                    </div>
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
};